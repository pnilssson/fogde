use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Deserialize;

use super::{Process, tail};

const LONGEST: usize = 200;

#[derive(Deserialize)]
struct Session {
    #[serde(rename = "sessionId")]
    id: String,
    cwd: PathBuf,
    #[serde(rename = "startedAt")]
    started: u64,
}

#[derive(Deserialize)]
struct Line {
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "isSidechain", default)]
    sidechain: bool,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    model: Option<String>,
}

pub(super) fn model(home: &Path, processes: &[Process]) -> Option<String> {
    let session = processes
        .iter()
        .find_map(|&process| session(home, process))?;
    let transcript = transcript(&home.join("projects"), &session)?;

    tail::last(&transcript, reply).map(|id| name(&id))
}

fn session(home: &Path, process: Process) -> Option<Session> {
    let file = home.join("sessions").join(format!("{}.json", process.pid));
    let session: Session = serde_json::from_slice(&fs::read(file).ok()?).ok()?;
    let started = SystemTime::UNIX_EPOCH + Duration::from_millis(session.started);

    process.before(started).then_some(session)
}

fn transcript(projects: &Path, session: &Session) -> Option<PathBuf> {
    let file = format!("{}.jsonl", session.id);
    let folder = folder(&session.cwd.to_string_lossy());
    if folder.len() <= LONGEST {
        return Some(projects.join(folder).join(file));
    }

    let prefix = format!("{}-", &folder[..LONGEST]);
    fs::read_dir(projects)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .map(|entry| entry.path().join(&file))
        .find(|path| path.is_file())
}

fn folder(cwd: &str) -> String {
    cwd.encode_utf16()
        .map(|unit| match char::from_u32(u32::from(unit)) {
            Some(letter) if letter.is_ascii_alphanumeric() => letter,
            _ => '-',
        })
        .collect()
}

fn reply(line: &str) -> Option<String> {
    if !line.contains(r#""assistant""#) {
        return None;
    }

    let line: Line = serde_json::from_str(line).ok()?;
    if line.kind != "assistant" || line.sidechain {
        return None;
    }

    line.message?.model.filter(|model| !model.starts_with('<'))
}

fn name(id: &str) -> String {
    pretty(id).unwrap_or_else(|| id.to_owned())
}

fn pretty(id: &str) -> Option<String> {
    let mut parts = id.strip_prefix("claude-")?.split('-');
    let family = parts.next()?;
    if family.is_empty() || !family.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return None;
    }

    let mut version: Vec<&str> = parts.collect();
    if version
        .last()
        .is_some_and(|part| part.len() == 8 && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        version.pop();
    }
    let numbers = version.iter().all(|part| {
        (1..=2).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_digit())
    });
    if version.is_empty() || !numbers {
        return None;
    }

    Some(format!(
        "{}{} {}",
        family[..1].to_ascii_uppercase(),
        &family[1..],
        version.join(".")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = r#"{"pid":9816,"sessionId":"f22bed96-c18a-49ab-abf0-66b04849a03a","cwd":"/repos/app","startedAt":1793030948964,"procStart":"134375045481041528","version":"2.1.310","peerProtocol":1,"peerFeatures":["notify_idle","artifact_yield"],"kind":"interactive","entrypoint":"cli","pidDomain":"win32:dev-pc","name":"app-7a","nameSource":"derived","nameSince":1793030948965,"status":"idle","updatedAt":1793030949079,"statusUpdatedAt":1793030949079}"#;
    const REPLY: &str = r#"{"parentUuid":"8de3e0cd-9f26-43d8-ab1d-959a3f684417","isSidechain":false,"message":{"model":"claude-opus-5-5","id":"msg_011CgR93GV6J6Aa78St3gc49","type":"message","role":"assistant","content":[{"type":"text","text":"ok"}],"container":null,"stop_reason":"end_turn","stop_sequence":null,"stop_details":null,"usage":{"input_tokens":2,"cache_creation_input_tokens":31880,"cache_read_input_tokens":32277,"output_tokens":4,"service_tier":"standard"},"context_management":null},"apiBlockIndex":0,"requestId":"req_011CgR93GDTfxxs4vvckQbVZ","requestedModel":"claude-opus-5-5","type":"assistant","uuid":"429be937-e7f1-4871-94a8-72f0d65d1474","timestamp":"2026-10-26T16:09:35.147Z","advisorModel":"claude-opus-5-5","effort":"high","perTurnEffort":"high","userType":"external","entrypoint":"cli","cwd":"/repos/app","sessionId":"f22bed96-c18a-49ab-abf0-66b04849a03a","version":"2.1.310","gitBranch":"main"}"#;
    const PROMPT: &str = r#"{"parentUuid":null,"isSidechain":false,"type":"user","message":{"role":"user","content":"Reply with only the word ok"},"uuid":"8de3e0cd-9f26-43d8-ab1d-959a3f684417","timestamp":"2026-10-26T16:09:31.112Z","cwd":"/repos/app","sessionId":"f22bed96-c18a-49ab-abf0-66b04849a03a"}"#;
    const TITLE: &str = r#"{"type":"ai-title","aiTitle":"Reply with ok","sessionId":"f22bed96-c18a-49ab-abf0-66b04849a03a"}"#;

    fn process(pid: u32, seconds: u64) -> Process {
        Process {
            pid,
            started: SystemTime::UNIX_EPOCH + Duration::from_secs(seconds),
        }
    }

    fn running(pids: &[u32]) -> Vec<Process> {
        pids.iter()
            .map(|&pid| process(pid, 1_793_030_940))
            .collect()
    }

    fn home(transcript: &[&str]) -> tempfile::TempDir {
        let home = tempfile::tempdir().unwrap();
        let sessions = home.path().join("sessions");
        let project = home.path().join("projects").join("-repos-app");
        fs::create_dir_all(&sessions).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::write(sessions.join("9816.json"), SESSION).unwrap();
        fs::write(
            project.join("f22bed96-c18a-49ab-abf0-66b04849a03a.jsonl"),
            transcript.join("\n") + "\n",
        )
        .unwrap();

        home
    }

    #[test]
    fn reads_the_model_of_the_last_reply() {
        let home = home(&[PROMPT, REPLY, TITLE]);

        assert_eq!(
            model(home.path(), &running(&[4100, 9816])).as_deref(),
            Some("Opus 5.5")
        );
    }

    #[test]
    fn skips_subagent_and_synthetic_replies() {
        let subagent = REPLY
            .replace(r#""isSidechain":false"#, r#""isSidechain":true"#)
            .replace("claude-opus-5-5", "claude-haiku-4-5-20251001");
        let synthetic = REPLY.replace("claude-opus-5-5", "<synthetic>");
        let home = home(&[REPLY, &subagent, &synthetic]);

        assert_eq!(
            model(home.path(), &running(&[9816])).as_deref(),
            Some("Opus 5.5")
        );
    }

    #[test]
    fn shows_nothing_before_the_first_reply() {
        let home = home(&[PROMPT]);

        assert_eq!(model(home.path(), &running(&[9816])), None);
    }

    #[test]
    fn ignores_processes_without_a_session() {
        let home = home(&[REPLY]);

        assert_eq!(model(home.path(), &running(&[4100])), None);
    }

    #[test]
    fn ignores_a_session_left_by_an_earlier_process_with_the_same_id() {
        let home = home(&[REPLY]);

        assert_eq!(model(home.path(), &[process(9816, 1_793_031_000)]), None);
    }

    #[test]
    fn names_the_project_folder_as_claude_does() {
        assert_eq!(folder(r"C:\Users\dev\app"), "C--Users-dev-app");
        assert_eq!(folder("/home/dev/my app.v2"), "-home-dev-my-app-v2");
        assert_eq!(folder("/home/dev/åpp"), "-home-dev--pp");
    }

    #[test]
    fn finds_a_shortened_project_folder() {
        let home = tempfile::tempdir().unwrap();
        let cwd = format!("/repos/{}", "a".repeat(250));
        let session = Session {
            id: "f22bed96".into(),
            cwd: PathBuf::from(&cwd),
            started: 0,
        };
        let shortened = format!("{}-1x2y3z", &folder(&cwd)[..LONGEST]);
        let project = home.path().join(shortened);
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("f22bed96.jsonl"), "").unwrap();

        assert_eq!(
            transcript(home.path(), &session),
            Some(project.join("f22bed96.jsonl"))
        );
    }

    #[test]
    fn names_models_as_claude_code_shows_them() {
        assert_eq!(name("claude-opus-5-5"), "Opus 5.5");
        assert_eq!(name("claude-sonnet-4-5-20250929"), "Sonnet 4.5");
        assert_eq!(name("claude-opus-4-20250514"), "Opus 4");
        assert_eq!(
            name("claude-3-5-sonnet-20241022"),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(name("glm-4.6"), "glm-4.6");
    }
}
