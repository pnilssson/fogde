use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use protocol::usage::Window;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{span, statusline, window};
use crate::shared::Shared;
use crate::terminals::path_with;

const FILE: &str = "FOGDE_USAGE";

#[derive(Default, Deserialize)]
struct Status {
    rate_limits: Option<Limits>,
    workspace: Option<Workspace>,
    cwd: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Workspace {
    project_dir: Option<PathBuf>,
}

#[derive(Deserialize, Serialize)]
struct Limits {
    five_hour: Option<Limit>,
    seven_day: Option<Limit>,
}

#[derive(Deserialize, Serialize)]
struct Limit {
    used_percentage: f64,
    resets_at: i64,
}

pub async fn launch(
    shared: &Shared,
    folder: &Path,
    env: &mut BTreeMap<String, String>,
) -> Vec<String> {
    if !shared.settings.shown().await.unwrap_or_default().claude {
        return Vec::new();
    }
    let Some(program) = program() else {
        return Vec::new();
    };
    let Some(name) = program.file_name().and_then(OsStr::to_str) else {
        return Vec::new();
    };
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        tracing::warn!(
            name,
            "Claude usage needs a program name without spaces or symbols"
        );
        return Vec::new();
    }

    env.insert("PATH".to_owned(), path_with(&program));
    env.insert(FILE.to_owned(), file(shared).to_string_lossy().into_owned());
    let own = statusline::find(folder).await;
    let mut line = serde_json::json!({ "type": "command", "command": format!("{name} usage") });
    if let Some(padding) = own.as_ref().and_then(|own| own.padding) {
        line["padding"] = padding.into();
    }
    if let Some(interval) = own.as_ref().and_then(|own| own.refresh_interval) {
        line["refreshInterval"] = interval.into();
    }

    vec![
        "--settings".to_owned(),
        serde_json::json!({ "statusLine": line }).to_string(),
    ]
}

pub async fn report() {
    let Some(path) = std::env::var_os(FILE).map(PathBuf::from) else {
        return;
    };
    let mut input = Vec::new();
    if tokio::io::stdin().read_to_end(&mut input).await.is_err() {
        return;
    }
    let status: Status = serde_json::from_slice(&input).unwrap_or_default();

    if let Some(limits) = &status.rate_limits {
        let _ = save(&path, limits).await;
    }

    let Some(project) = status
        .workspace
        .and_then(|workspace| workspace.project_dir)
        .or(status.cwd)
    else {
        return;
    };
    let Some(own) = statusline::find(&project).await else {
        return;
    };
    let Ok(output) = statusline::run(&own.command, &input, FILE).await else {
        return;
    };

    let mut stdout = tokio::io::stdout();
    let _ = stdout.write_all(&output).await;
    let _ = stdout.flush().await;
}

pub fn file(shared: &Shared) -> PathBuf {
    shared.home().join(".app").join("usage").join("claude.json")
}

pub async fn read(path: &Path) -> Option<Vec<Window>> {
    let bytes = fs::read(path).await.ok()?;
    let limits: Limits = serde_json::from_slice(&bytes).ok()?;

    Some(windows(limits))
}

fn program() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(image) = std::env::var_os("APPIMAGE") {
        return Some(PathBuf::from(image));
    }

    let exe = std::env::current_exe().ok()?;
    #[cfg(unix)]
    if let Some(link) = crate::daemon::link(&exe) {
        return Some(link);
    }

    Some(exe)
}

#[cfg(test)]
fn limits(status: &[u8]) -> Option<Limits> {
    serde_json::from_slice::<Status>(status).ok()?.rate_limits
}

fn windows(limits: Limits) -> Vec<Window> {
    [(300, limits.five_hour), (10080, limits.seven_day)]
        .into_iter()
        .filter_map(|(minutes, limit)| {
            limit.map(|limit| {
                window(
                    span(minutes),
                    100.0 - limit.used_percentage,
                    limit.resets_at,
                )
            })
        })
        .collect()
}

async fn save(path: &Path, limits: &Limits) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).await?;
    }
    let partial = path.with_extension(format!("{}.partial", std::process::id()));
    fs::write(&partial, serde_json::to_vec(limits)?).await?;

    if let Err(error) = fs::rename(&partial, path).await {
        let _ = fs::remove_file(&partial).await;
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use protocol::usage::Span;

    use super::*;

    const STATUS: &[u8] = br#"{"cwd":"/repos/app","session_id":"abc123","transcript_path":"/home/dev/.claude/projects/app/abc123.jsonl","model":{"id":"claude-opus-5-5","display_name":"Opus"},"workspace":{"current_dir":"/repos/app","project_dir":"/repos/app"},"version":"2.1.300","output_style":{"name":"default"},"cost":{"total_cost_usd":0.01234,"total_duration_ms":45000,"total_api_duration_ms":2300,"total_lines_added":156,"total_lines_removed":23},"context_window":{"total_input_tokens":15234,"total_output_tokens":4521,"context_window_size":200000,"used_percentage":8,"remaining_percentage":92},"exceeds_200k_tokens":false,"rate_limits":{"five_hour":{"used_percentage":23.5,"resets_at":1738425600},"seven_day":{"used_percentage":41.2,"resets_at":1738857600}}}"#;

    #[tokio::test]
    async fn keeps_the_limits_from_a_status() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("usage").join("claude.json");

        save(&path, &limits(STATUS).unwrap()).await.unwrap();
        let saved: Limits = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();

        assert_eq!(
            windows(saved),
            [
                Window {
                    span: Span::Session,
                    left: 76.5,
                    resets: 1_738_425_600,
                },
                Window {
                    span: Span::Week,
                    left: 58.8,
                    resets: 1_738_857_600,
                },
            ]
        );
    }

    #[test]
    fn skips_a_status_without_limits() {
        let first = br#"{"cwd":"/repos/app","model":{"id":"claude-opus-5-5","display_name":"Opus"},"context_window":{"used_percentage":null}}"#;

        assert!(limits(first).is_none());
        assert!(limits(b"not json").is_none());
    }

    #[test]
    fn keeps_a_single_window() {
        let status =
            br#"{"rate_limits":{"seven_day":{"used_percentage":100,"resets_at":1738857600}}}"#;

        assert_eq!(
            windows(limits(status).unwrap()),
            [Window {
                span: Span::Week,
                left: 0.0,
                resets: 1_738_857_600,
            }]
        );
    }
}
