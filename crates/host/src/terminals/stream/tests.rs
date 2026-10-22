use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};

use super::*;

const FRAMES: u16 = 200;

#[cfg(unix)]
fn herdr(dir: &Path, name: &str, frames: u16, idle: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(name);
    let wait = if idle { "read -r line\n" } else { "" };
    let script = format!(
        r#"#!/bin/sh
i=1
while [ "$i" -le {frames} ]; do
  printf '{{"type":"terminal.frame","seq":%d,"width":%d,"height":24,"full":false,"bytes":"aGk="}}\n' "$i" "$i"
  i=$((i + 1))
done
{wait}"#
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();

    path
}

#[cfg(windows)]
fn herdr(dir: &Path, name: &str, frames: u16, idle: bool) -> PathBuf {
    let path = dir.join(format!("{name}.cmd"));
    let wait = if idle { "set /p line=\r\n" } else { "" };
    let script = format!(
        "@echo off\r\nfor /L %%i in (1,1,{frames}) do echo {{\"type\":\"terminal.frame\",\"seq\":%%i,\"width\":%%i,\"height\":24,\"full\":false,\"bytes\":\"aGk=\"}}\r\n{wait}"
    );
    std::fs::write(&path, script).unwrap();

    path
}

fn open(dir: &Path, name: &str, frames: u16, idle: bool) -> herdr_client::Terminal {
    herdr_client::Terminal::open(
        &herdr(dir, name, frames, idle),
        &dir.join("herdr.sock"),
        "w1:p1",
        80,
        24,
    )
    .unwrap()
}

fn keys(dir: &Path) -> mpsc::UnboundedSender<Key> {
    press(Client::new(&dir.join("herdr.sock")), "w1:p1".to_owned())
}

fn wire(message: Message) -> Value {
    serde_json::to_value(message).unwrap()
}

#[tokio::test]
async fn a_client_that_falls_behind_holds_herdr_back_without_losing_frames() {
    let dir = tempfile::tempdir().unwrap();
    let stream = open(dir.path(), "herdr", FRAMES, false);
    let (_controls, receiver) = mpsc::unbounded_channel();
    let (frames, mut queued) = mpsc::channel(4);
    tokio::spawn(watch(
        stream,
        keys(dir.path()),
        "w1".to_owned(),
        receiver,
        frames,
    ));

    tokio::time::timeout(Duration::from_secs(10), async {
        while queued.len() < 4 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(queued.len(), 4);

    for cols in 1..=FRAMES {
        let frame = wire(queued.recv().await.unwrap());
        assert_eq!(frame["method"], "terminals.frame");
        assert_eq!(frame["params"]["cols"], cols);
    }
    assert_eq!(
        wire(queued.recv().await.unwrap()),
        json!({"jsonrpc": "2.0", "method": "terminals.ended", "params": {"id": "w1", "error": null}})
    );
}

#[tokio::test]
async fn idle_terminals_never_hold_back_a_busy_one() {
    let dir = tempfile::tempdir().unwrap();
    let (frames, mut queued) = mpsc::channel(2);
    let mut controls = Vec::new();
    for name in ["idle1", "idle2", "idle3"] {
        let (sender, receiver) = mpsc::unbounded_channel();
        controls.push(sender);
        let stream = open(dir.path(), name, 0, true);
        tokio::spawn(watch(
            stream,
            keys(dir.path()),
            name.to_owned(),
            receiver,
            frames.clone(),
        ));
    }
    tokio::time::sleep(Duration::from_millis(300)).await;

    let (_busy, receiver) = mpsc::unbounded_channel();
    let stream = open(dir.path(), "busy", 3, true);
    tokio::spawn(watch(
        stream,
        keys(dir.path()),
        "busy".to_owned(),
        receiver,
        frames,
    ));

    let arrived = tokio::time::timeout(Duration::from_secs(10), async {
        let mut cols = Vec::new();
        while cols.len() < 3 {
            let frame = wire(queued.recv().await.unwrap());
            assert_eq!(frame["params"]["id"], "busy");
            cols.push(frame["params"]["cols"].clone());
        }
        cols
    })
    .await;
    assert_eq!(arrived.unwrap(), [json!(1), json!(2), json!(3)]);
}

#[tokio::test]
async fn a_key_that_fails_to_send_leaves_the_terminal_open() {
    let dir = tempfile::tempdir().unwrap();
    let stream = open(dir.path(), "herdr", 1, true);
    let (controls, receiver) = mpsc::unbounded_channel();
    let (frames, mut queued) = mpsc::channel(4);
    tokio::spawn(watch(
        stream,
        keys(dir.path()),
        "w1".to_owned(),
        receiver,
        frames,
    ));
    let frame = wire(queued.recv().await.unwrap());
    assert_eq!(frame["method"], "terminals.frame");

    assert!(controls.send(Control::Key(Key::ShiftEnter)).is_ok());
    tokio::time::sleep(Duration::from_millis(300)).await;

    assert!(queued.try_recv().is_err());
}
