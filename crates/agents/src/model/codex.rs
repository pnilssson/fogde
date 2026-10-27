use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, SqliteConnection};

use super::{blocking, tail};

const STATE: &str = "state_5.sqlite";
const LOGS: &str = "logs_2.sqlite";

#[derive(Deserialize)]
struct Line {
    #[serde(rename = "type")]
    kind: String,
    payload: Turn,
}

#[derive(Deserialize)]
struct Turn {
    model: Option<String>,
}

pub(super) async fn model(home: &Path, pids: &[u32], started: SystemTime) -> Option<String> {
    let rollout = match rollout(home, pids, started).await {
        Ok(rollout) => rollout?,
        Err(error) => {
            tracing::warn!(%error, "could not read Codex's sessions");
            return None;
        }
    };

    blocking(move || tail::last(&rollout, turn)).await
}

async fn rollout(
    home: &Path,
    pids: &[u32],
    started: SystemTime,
) -> Result<Option<PathBuf>, sqlx::Error> {
    let (Ok(mut state), Ok(mut logs)) =
        (open(&home.join(STATE)).await, open(&home.join(LOGS)).await)
    else {
        return Ok(None);
    };
    let since = started.duration_since(UNIX_EPOCH).map_or(0, |since| {
        i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
    });

    let threads: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, rollout_path FROM threads
         WHERE updated_at >= ? AND archived = 0
         AND id NOT IN (SELECT child_thread_id FROM thread_spawn_edges)",
    )
    .bind(since)
    .fetch_all(&mut state)
    .await?;

    let mut owned: Option<(usize, i64, String)> = None;
    for (thread, rollout) in threads {
        let last: Option<(Option<String>, i64)> = sqlx::query_as(
            "SELECT process_uuid, id FROM logs WHERE thread_id = ?
             ORDER BY ts DESC, ts_nanos DESC, id DESC LIMIT 1",
        )
        .bind(&thread)
        .fetch_optional(&mut logs)
        .await?;

        let Some((Some(process), row)) = last else {
            continue;
        };
        let Some(depth) = owner(&process).and_then(|pid| pids.iter().position(|&id| id == pid))
        else {
            continue;
        };
        if owned
            .as_ref()
            .is_none_or(|(best, latest, _)| (depth, -row) < (*best, -*latest))
        {
            owned = Some((depth, row, rollout));
        }
    }

    Ok(owned.map(|(_, _, rollout)| PathBuf::from(rollout)))
}

async fn open(path: &Path) -> Result<SqliteConnection, sqlx::Error> {
    SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .connect()
        .await
}

fn owner(process: &str) -> Option<u32> {
    process
        .strip_prefix("pid:")?
        .split(':')
        .next()?
        .parse()
        .ok()
}

fn turn(line: &str) -> Option<String> {
    if !line.contains(r#""turn_context""#) {
        return None;
    }

    let line: Line = serde_json::from_str(line).ok()?;
    if line.kind != "turn_context" {
        return None;
    }

    line.payload.model
}

#[cfg(test)]
mod tests;
