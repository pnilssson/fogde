use std::fs;
use std::time::Duration;

use sqlx::Executor;

use super::*;

const TURN: &str = r#"{"timestamp":"2026-10-26T18:09:36.955Z","ordinal":9,"type":"turn_context","payload":{"turn_id":"01a17967-f6bd-7770-93d6-f6a3f51e06bb","root_turn_id":"01a17967-f6bd-7770-93d6-f6a3f51e06bb","disabled_plugin_ids":[],"cwd":"/repos/app","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"danger-full-access"},"model":"gpt-6.1-sol","comp_hash":"3000","collaboration_mode":{"mode":"default","settings":{"model":"gpt-6.1-sol","reasoning_effort":"high"}},"multi_agent_version":"v2","realtime_active":false,"effort":"high","summary":"auto"}}"#;
const TOKENS: &str = r#"{"timestamp":"2026-10-26T18:09:40.513Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":28660,"cached_input_tokens":13184,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":28665},"model_context_window":475000}}}"#;

const THREADS: &str = "CREATE TABLE threads (
    id TEXT PRIMARY KEY,
    rollout_path TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    source TEXT NOT NULL,
    cwd TEXT NOT NULL,
    archived INTEGER NOT NULL DEFAULT 0,
    model TEXT
);
CREATE TABLE thread_spawn_edges (
    parent_thread_id TEXT NOT NULL,
    child_thread_id TEXT NOT NULL PRIMARY KEY,
    status TEXT NOT NULL
)";
const LOG_TABLE: &str = "CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    ts_nanos INTEGER NOT NULL,
    level TEXT NOT NULL,
    target TEXT NOT NULL,
    feedback_log_body TEXT,
    module_path TEXT,
    file TEXT,
    line INTEGER,
    thread_id TEXT,
    process_uuid TEXT,
    estimated_bytes INTEGER NOT NULL DEFAULT 0
)";

const FIRST: &str = "01a17967-4818-7d60-8998-ad13ca350bdd";
const SECOND: &str = "01a17967-4818-78a0-8815-31bb3f20ce9f";
const FIRST_PROCESS: &str = "pid:40692:7f1c2a90-55e1-4f0e-9d2b-3c8a1e6b4d70";
const SECOND_PROCESS: &str = "pid:40640:2b9e4c1d-8a3f-4e6b-b1d0-5f7a9c3e2d18";
const STARTED: i64 = 1_793_038_170;

struct Codex {
    home: tempfile::TempDir,
    state: SqliteConnection,
    logs: SqliteConnection,
}

impl Codex {
    async fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let state = create(&home.path().join(STATE), THREADS).await;
        let logs = create(&home.path().join(LOGS), LOG_TABLE).await;

        Self { home, state, logs }
    }

    async fn thread(&mut self, id: &str, updated: i64, model: &str) {
        let rollout = self.home.path().join(format!("rollout-{id}.jsonl"));
        fs::write(
            &rollout,
            [TURN.replace("gpt-6.1-sol", model).as_str(), TOKENS].join("\n") + "\n",
        )
        .unwrap();

        sqlx::query(
            "INSERT INTO threads (id, rollout_path, created_at, updated_at, source, cwd)
             VALUES (?, ?, ?, ?, 'cli', '\\\\?\\C:\\repos\\app')",
        )
        .bind(id)
        .bind(rollout.to_string_lossy().into_owned())
        .bind(STARTED + 4)
        .bind(updated)
        .execute(&mut self.state)
        .await
        .unwrap();
    }

    async fn log(&mut self, ts: i64, thread: Option<&str>, process: &str) {
        sqlx::query(
            "INSERT INTO logs (ts, ts_nanos, level, target, thread_id, process_uuid)
             VALUES (?, 0, 'INFO', 'codex_core::session', ?, ?)",
        )
        .bind(ts)
        .bind(thread)
        .bind(process)
        .execute(&mut self.logs)
        .await
        .unwrap();
    }

    async fn model(&self, pid: u32) -> Option<String> {
        model(self.home.path(), &[pid], started()).await
    }
}

async fn create(path: &Path, schema: &'static str) -> SqliteConnection {
    let mut connection = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .connect()
        .await
        .unwrap();
    connection.execute(schema).await.unwrap();

    connection
}

fn started() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(STARTED.unsigned_abs())
}

#[tokio::test]
async fn tells_apart_two_sessions_started_together_in_one_folder() {
    let mut codex = Codex::new().await;
    codex.thread(FIRST, STARTED + 37, "gpt-6.1-sol").await;
    codex.thread(SECOND, STARTED + 35, "gpt-6-luna").await;
    codex.log(STARTED, None, FIRST_PROCESS).await;
    codex.log(STARTED + 4, Some(FIRST), FIRST_PROCESS).await;
    codex.log(STARTED + 4, Some(SECOND), SECOND_PROCESS).await;
    codex.log(STARTED + 37, Some(FIRST), FIRST_PROCESS).await;

    assert_eq!(codex.model(40692).await.as_deref(), Some("gpt-6.1-sol"));
    assert_eq!(codex.model(40640).await.as_deref(), Some("gpt-6-luna"));
}

#[tokio::test]
async fn follows_the_session_a_new_chat_starts() {
    let mut codex = Codex::new().await;
    let next = "01a17968-5b30-7012-ac00-d567c4835c3b";
    codex.thread(FIRST, STARTED + 37, "gpt-6.1-sol").await;
    codex.thread(next, STARTED + 82, "gpt-6-luna").await;
    codex.log(STARTED + 37, Some(FIRST), FIRST_PROCESS).await;
    codex.log(STARTED + 74, Some(next), FIRST_PROCESS).await;

    assert_eq!(codex.model(40692).await.as_deref(), Some("gpt-6-luna"));
}

#[tokio::test]
async fn shows_nothing_for_a_session_on_the_shared_server() {
    let mut codex = Codex::new().await;
    codex.thread(FIRST, STARTED + 37, "gpt-6.1-sol").await;
    codex
        .log(
            STARTED + 4,
            Some(FIRST),
            "pid:21248:43fc6dad-44e0-4aaa-9812-3fb31575f7bc",
        )
        .await;

    assert_eq!(codex.model(35864).await, None);
}

#[tokio::test]
async fn ignores_subagents_and_sessions_idle_since_before_the_process() {
    let mut codex = Codex::new().await;
    let child = "01a17967-c059-73e0-ae3e-f3a76a0b1dc1";
    codex.thread(FIRST, STARTED - 600, "gpt-6.1-sol").await;
    codex.thread(child, STARTED + 40, "gpt-6-luna").await;
    sqlx::query("INSERT INTO thread_spawn_edges VALUES (?, ?, 'running')")
        .bind(FIRST)
        .bind(child)
        .execute(&mut codex.state)
        .await
        .unwrap();
    codex.log(STARTED - 600, Some(FIRST), FIRST_PROCESS).await;
    codex.log(STARTED + 40, Some(child), FIRST_PROCESS).await;

    assert_eq!(codex.model(40692).await, None);
}

#[tokio::test]
async fn shows_nothing_without_codex_databases() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(model(home.path(), &[40692], started()).await, None);
}

#[tokio::test]
async fn prefers_the_terminal_session_over_a_nested_codex_run() {
    let mut codex = Codex::new().await;
    let nested = "01a17969-2c4d-7a10-9e3b-8f6d1a2b3c4e";
    codex.thread(FIRST, STARTED + 37, "gpt-6.1-sol").await;
    codex.thread(nested, STARTED + 60, "gpt-6-luna").await;
    codex.log(STARTED + 37, Some(FIRST), FIRST_PROCESS).await;
    codex
        .log(
            STARTED + 60,
            Some(nested),
            "pid:51200:9c4e2a7b-1d3f-4b8a-a6e5-0f2c7d9b3e41",
        )
        .await;

    assert_eq!(
        model(codex.home.path(), &[40692, 51200], started())
            .await
            .as_deref(),
        Some("gpt-6.1-sol")
    );
}
