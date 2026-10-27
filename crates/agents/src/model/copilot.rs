use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::{Process, tail};

#[derive(Deserialize)]
struct Line {
    #[serde(rename = "type")]
    kind: String,
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    usage: Usage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    model: Option<String>,
    model_display_name: Option<String>,
    initiator: Option<String>,
}

pub(super) fn model(home: &Path, processes: &[Process]) -> Option<String> {
    let sessions: Vec<_> = fs::read_dir(home.join("session-state"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    let session = processes
        .iter()
        .find_map(|&process| sessions.iter().find(|session| holds(session, process)))?;

    tail::last(&session.join("events.jsonl"), usage)
}

fn holds(session: &Path, process: Process) -> bool {
    let lock = session.join(format!("inuse.{}.lock", process.pid));

    fs::metadata(lock)
        .and_then(|metadata| metadata.modified())
        .is_ok_and(|locked| process.before(locked))
}

fn usage(line: &str) -> Option<String> {
    if !line.contains(r#""session.usage_record""#) {
        return None;
    }

    let line: Line = serde_json::from_str(line).ok()?;
    let usage = line.data.usage;
    if line.kind != "session.usage_record"
        || usage
            .initiator
            .as_deref()
            .is_some_and(|initiator| initiator != "user")
    {
        return None;
    }

    usage.model_display_name.or(usage.model)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;

    const START: &str = r#"{"type":"session.start","data":{"sessionId":"80e1a74f-5cda-4a19-818c-9076b8d366c4","version":1,"producer":"copilot-agent","copilotVersion":"1.0.101","startTime":"2026-10-26T16:12:34.259Z","contextTier":null,"context":{"cwd":"/repos/app"},"alreadyInUse":false,"remoteSteerable":false},"id":"ade77ca9-ff5c-4ba4-82b3-a587f42d7d05","timestamp":"2026-10-26T16:12:34.407Z","parentId":null}"#;
    const USAGE: &str = r#"{"type":"session.usage_record","data":{"usage":{"model":"mai-code-1.1-flash","provider":{"id":"copilot","kind":"copilot","label":"GitHub Copilot"},"modelDisplayName":"MAI-Code-1.1-Flash","inputTokens":15588,"outputTokens":5,"cacheReadTokens":1408,"cacheWriteTokens":0,"reasoningTokens":0,"cost":1,"duration":1774,"initiator":"user","interactionType":"conversation-agent","isByok":false,"isAuto":true,"maxPromptTokens":128000,"transport":"websocket","apiEndpoint":"ws:/responses","reasoningEffort":"medium","finishReason":"stop","contentFilterTriggered":false}},"id":"59bc663a-389c-489e-b5e3-c215dc31dab3","timestamp":"2026-10-26T16:13:02.371Z","parentId":"d56756ba-04ec-4cb8-95d2-6f2d4109376f"}"#;
    const REPLY: &str = r#"{"type":"assistant.message","data":{"messageId":"c91a9241-b5c7-4dcc-ad80-694763532ad5","model":"mai-code-1.1-flash","providerId":"copilot","content":"ok","toolRequests":[],"turnId":"0","phase":"final_answer"},"id":"0b5f5d3e-8c34-4c55-a0f5-8f5b0c8f1e21","timestamp":"2026-10-26T16:13:02.380Z","parentId":"59bc663a-389c-489e-b5e3-c215dc31dab3"}"#;

    fn running(pids: &[u32]) -> Vec<Process> {
        pids.iter()
            .map(|&pid| Process {
                pid,
                started: SystemTime::now() - Duration::from_secs(60),
            })
            .collect()
    }

    fn home(lock: &str, events: &[&str]) -> tempfile::TempDir {
        let home = tempfile::tempdir().unwrap();
        let other = home.path().join("session-state").join("388ad099");
        let session = home.path().join("session-state").join("80e1a74f");
        fs::create_dir_all(&other).unwrap();
        fs::create_dir_all(&session).unwrap();
        fs::write(
            other.join("events.jsonl"),
            USAGE.replace("MAI-Code-1.1-Flash", "GPT-6.1"),
        )
        .unwrap();
        fs::write(session.join(lock), "").unwrap();
        fs::write(session.join("events.jsonl"), events.join("\n") + "\n").unwrap();

        home
    }

    #[test]
    fn reads_the_model_of_the_session_the_process_holds() {
        let home = home("inuse.18660.lock", &[START, USAGE, REPLY]);

        assert_eq!(
            model(home.path(), &running(&[5332, 18660])).as_deref(),
            Some("MAI-Code-1.1-Flash")
        );
    }

    #[test]
    fn prefers_the_session_of_the_foreground_process_over_its_children() {
        let home = home("inuse.18660.lock", &[START, USAGE]);
        let child = home.path().join("session-state").join("388ad099");
        fs::write(child.join("inuse.20112.lock"), "").unwrap();

        assert_eq!(
            model(home.path(), &running(&[18660, 20112])).as_deref(),
            Some("MAI-Code-1.1-Flash")
        );
        assert_eq!(
            model(home.path(), &running(&[20112, 18660])).as_deref(),
            Some("GPT-6.1")
        );
    }

    #[test]
    fn skips_usage_the_agent_started_itself() {
        let background = USAGE
            .replace(r#""initiator":"user""#, r#""initiator":"agent""#)
            .replace("MAI-Code-1.1-Flash", "GPT-4o mini");
        let home = home("inuse.18660.lock", &[START, USAGE, &background]);

        assert_eq!(
            model(home.path(), &running(&[18660])).as_deref(),
            Some("MAI-Code-1.1-Flash")
        );
    }

    #[test]
    fn shows_nothing_before_the_first_reply_or_for_other_processes() {
        let home = home("inuse.18660.lock", &[START]);

        assert_eq!(model(home.path(), &running(&[18660])), None);
        assert_eq!(model(home.path(), &running(&[4100])), None);
    }

    #[test]
    fn ignores_a_lock_left_by_an_earlier_process_with_the_same_id() {
        let home = home("inuse.18660.lock", &[START, USAGE]);
        let later = Process {
            pid: 18660,
            started: SystemTime::now() + Duration::from_secs(60),
        };

        assert_eq!(model(home.path(), &[later]), None);
    }
}
