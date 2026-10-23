use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const WAIT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct Settings {
    #[serde(rename = "statusLine")]
    line: Option<Line>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub command: String,
    pub padding: Option<u32>,
    pub refresh_interval: Option<u32>,
}

pub async fn find(project: &Path) -> Option<Line> {
    let mut files = vec![
        project.join(".claude").join("settings.local.json"),
        project.join(".claude").join("settings.json"),
    ];
    files.extend(config().map(|dir| dir.join("settings.json")));

    first(&files).await
}

pub async fn run(line: &str, status: &[u8], hide: &str) -> io::Result<Vec<u8>> {
    within(WAIT, line, status, hide).await
}

async fn within(wait: Duration, line: &str, status: &[u8], hide: &str) -> io::Result<Vec<u8>> {
    let mut child = shell(line)
        .env_remove(hide)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(status).await;
    }

    let output = tokio::time::timeout(wait, child.wait_with_output())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "the status line took too long"))??;

    Ok(output.stdout)
}

async fn first(files: &[PathBuf]) -> Option<Line> {
    for file in files {
        let Ok(bytes) = fs::read(file).await else {
            continue;
        };
        if let Ok(Settings { line: Some(line) }) = serde_json::from_slice(&bytes) {
            return Some(line);
        }
    }

    None
}

fn config() -> Option<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".claude")))
}

#[cfg(windows)]
fn shell(line: &str) -> Command {
    if let Some(bash) = git_bash() {
        let mut command = process::hidden(bash);
        command.arg("-c").arg(line);
        return command;
    }

    let mut command = process::hidden(process::find("pwsh").unwrap_or_else(|| "powershell".into()));
    command.args(["-NoProfile", "-Command", line]);
    command
}

#[cfg(windows)]
fn git_bash() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CLAUDE_CODE_GIT_BASH_PATH") {
        return Some(PathBuf::from(path));
    }

    process::find("git")?
        .ancestors()
        .skip(1)
        .take(3)
        .map(|dir| dir.join("bin").join("bash.exe"))
        .find(|bash| bash.is_file())
}

#[cfg(not(windows))]
fn shell(line: &str) -> Command {
    let mut command = process::hidden("sh");
    command.arg("-c").arg(line);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn takes_the_closest_status_line() {
        let dir = tempfile::tempdir().unwrap();
        let local = dir.path().join("settings.local.json");
        let project = dir.path().join("settings.json");
        let user = dir.path().join("user.json");
        std::fs::write(&local, r#"{"permissions":{"allow":["Bash(npm test)"]}}"#).unwrap();
        std::fs::write(
            &project,
            r#"{"statusLine":{"type":"command","command":"~/.claude/statusline.sh","padding":2}}"#,
        )
        .unwrap();
        std::fs::write(
            &user,
            r#"{"statusLine":{"type":"command","command":"pwsh -NoProfile -File C:/Users/dev/.claude/statusline.ps1","refreshInterval":5},"enabledPlugins":{}}"#,
        )
        .unwrap();

        assert_eq!(
            first(&[local.clone(), project, user.clone()]).await,
            Some(Line {
                command: "~/.claude/statusline.sh".to_owned(),
                padding: Some(2),
                refresh_interval: None,
            })
        );
        assert_eq!(
            first(&[local, dir.path().join("missing.json"), user]).await,
            Some(Line {
                command: "pwsh -NoProfile -File C:/Users/dev/.claude/statusline.ps1".to_owned(),
                padding: None,
                refresh_interval: Some(5),
            })
        );
    }

    #[tokio::test]
    async fn passes_the_status_to_the_status_line() {
        let output = run("cat", br#"{"model":{"display_name":"Opus"}}"#, "FOGDE_TEST")
            .await
            .unwrap();

        assert_eq!(output, br#"{"model":{"display_name":"Opus"}}"#);
    }

    #[tokio::test]
    async fn stops_a_status_line_that_hangs() {
        let started = std::time::Instant::now();
        let error = within(Duration::from_millis(500), "sleep 3", b"{}", "FOGDE_TEST")
            .await
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
