use serde_json::json;

use super::*;
use crate::fake::{Core, connected};
use crate::wire::{CREDIT, DATA};

const TITLE: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"title":"app task"},"seq":7,"terminal":"t2","type":"title"}}"#;
const WORKING: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"records":{},"reported":true,"status":{"app":"demo","state":"working"}},"seq":8,"terminal":"t2","type":"status"}}"#;
const FOREGROUND: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"argv":["C:\\Users\\dev\\.local\\bin\\claude.EXE"],"cwd":"C:\\Users\\dev\\app","name":"claude.EXE","pid":35980},"seq":9,"terminal":"t2","type":"foreground"}}"#;
const DONE: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"records":{},"reported":true,"status":{"state":"done"}},"seq":10,"terminal":"t2","type":"status"}}"#;
const TURN: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"state":"done","turn":1},"seq":11,"terminal":"t2","type":"turn"}}"#;
const SIZE: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"cols":200,"rows":120},"seq":12,"terminal":"t2","type":"size"}}"#;
const HEARTBEAT: &str = r#"{"jsonrpc":"2.0","method":"heartbeat","params":{"seq":12}}"#;
const FINISHED: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":{"exit_code":1,"finished_ms":1793550537958,"reason":"killed","signal":null},"seq":13,"terminal":"t2","type":"finished"}}"#;

async fn subscribed(core: &mut Core, connection: &Connection) -> Events {
    let serving = async {
        let request = core.request().await;
        assert_eq!(request["method"], "events.subscribe");
        core.reply(
            &request["id"],
            json!({"channel": 2, "seq": 6, "terminals": [
                {"address": "t2", "argv": ["powershell"], "cwd": "C:\\Users\\dev\\app", "labels": {"fogde": "Claude"}, "state": "live", "turn": 0},
            ]}),
        )
        .await;
    };
    let (events, ()) = tokio::join!(connection.events(), serving);

    events.unwrap()
}

async fn lines(core: &mut Core, lines: &[&str]) {
    let mut bytes = lines.join("\n");
    bytes.push('\n');
    let (first, rest) = bytes.as_bytes().split_at(bytes.len() / 3);
    core.send(2, DATA, first).await;
    core.send(2, DATA, rest).await;
}

#[tokio::test]
async fn follows_a_terminal_through_a_turn() {
    let (connection, mut core) = connected().await;
    let mut events = subscribed(&mut core, &connection).await;

    lines(
        &mut core,
        &[TITLE, WORKING, FOREGROUND, HEARTBEAT, DONE, TURN, SIZE],
    )
    .await;
    let mut changes = Vec::new();
    for _ in 0..5 {
        changes.push(events.changed().await.unwrap());
    }

    let terminal = Change::Terminal("t2".into());
    assert_eq!(
        changes,
        [
            terminal.clone(),
            terminal.clone(),
            terminal.clone(),
            terminal,
            Change::Turn("t2".into())
        ]
    );
    let t2 = &events.terminals()["t2"];
    assert_eq!(t2.title, "app task");
    assert_eq!(
        t2.status.as_ref().map(|status| status.state.as_str()),
        Some("done")
    );
    assert_eq!(t2.turn, 1);
    assert_eq!(
        t2.foreground.as_ref().map(|process| process.pid),
        Some(35980)
    );
    assert_eq!(t2.labels["fogde"], "Claude");
    let credit = core.frame().await;
    assert_eq!((credit.channel, credit.kind), (2, CREDIT));
}

#[tokio::test]
async fn drops_a_terminal_once_it_finishes() {
    let (connection, mut core) = connected().await;
    let mut events = subscribed(&mut core, &connection).await;

    lines(&mut core, &[FINISHED]).await;

    assert_eq!(
        events.changed().await.unwrap(),
        Change::Terminal("t2".into())
    );
    assert!(events.terminals().is_empty());
}

#[tokio::test]
async fn starts_over_from_a_fresh_snapshot_after_falling_behind() {
    let (connection, mut core) = connected().await;
    let mut events = subscribed(&mut core, &connection).await;
    let missed = json!({"jsonrpc": "2.0", "method": "missed", "params": {"seq": 40, "terminals": [
        {"address": "t9", "argv": ["/bin/zsh"], "cwd": "/repos/app", "state": "live", "turn": 3},
    ]}})
    .to_string();

    lines(&mut core, &[&missed, TITLE, FINISHED]).await;

    assert_eq!(events.changed().await.unwrap(), Change::All);
    core.send(2, DATA, format!("{TITLE}\n").as_bytes()).await;
    let stale = tokio::time::timeout(std::time::Duration::from_millis(100), events.changed()).await;
    assert!(
        stale.is_err(),
        "events from before the snapshot were applied"
    );
    assert_eq!(events.terminals().keys().collect::<Vec<_>>(), ["t9"]);
}

#[tokio::test]
async fn reports_that_herdr_ended_the_events() {
    let (connection, mut core) = connected().await;
    let mut events = subscribed(&mut core, &connection).await;

    core.send(2, crate::wire::CLOSE, br#"{"reason":"server_stop"}"#)
        .await;

    assert!(matches!(events.changed().await, Err(Error::Ended(reason)) if reason == "server_stop"));
}
