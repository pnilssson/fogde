use std::future;
use std::pin::Pin;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::rpc::Message;
use protocol::terminals::{
    EndedEvent, FrameEvent, Scroll, ScrollTerminalParams, Size, TerminalEnded, TerminalFrame,
};
use tokio::sync::mpsc::{self, OwnedPermit, error::SendError};

use crate::rpc::notification;

pub enum Control {
    Input(Vec<u8>),
    Resize(Size),
    Scroll(ScrollTerminalParams),
}

type Slot = Pin<Box<dyn Future<Output = Result<OwnedPermit<Message>, SendError<()>>> + Send>>;

pub async fn watch(
    mut stream: herdr_client::Terminal,
    id: String,
    mut controls: mpsc::UnboundedReceiver<Control>,
    frames: mpsc::Sender<Message>,
) {
    let mut pending = None;
    let mut slot: Option<Slot> = None;

    let error = loop {
        tokio::select! {
            permit = reserved(&mut slot) => {
                slot = None;
                match (permit, pending.take()) {
                    (Ok(permit), Some(message)) => {
                        permit.send(message);
                    }
                    (Ok(_), None) => {}
                    (Err(_), _) => return,
                }
            }
            frame = stream.next(), if pending.is_none() => match frame {
                Ok(frame) => {
                    pending = notification::<FrameEvent>(TerminalFrame {
                        id: id.clone(),
                        cols: frame.width,
                        rows: frame.height,
                        full: frame.full,
                        data: STANDARD.encode(frame.bytes),
                    });
                    if pending.is_some() {
                        slot = Some(Box::pin(frames.clone().reserve_owned()));
                    }
                }
                Err(error) => break error,
            },
            control = controls.recv() => {
                let sent = match control {
                    Some(Control::Input(bytes)) => stream.input(&bytes).await,
                    Some(Control::Resize(size)) => stream.resize(size.cols, size.rows).await,
                    Some(Control::Scroll(scroll)) => {
                        let direction = match scroll.direction {
                            Scroll::Up => herdr_client::Scroll::Up,
                            Scroll::Down => herdr_client::Scroll::Down,
                        };
                        stream.scroll(direction, scroll.lines, scroll.column, scroll.row).await
                    }
                    None => return,
                };
                if let Err(error) = sent {
                    break error;
                }
            }
        }
    };

    tracing::info!(%error, %id, "terminal stream ended");
    drop(slot);
    let ended = notification::<EndedEvent>(TerminalEnded {
        id,
        error: match error {
            herdr_client::Error::Closed => None,
            error => Some(error.to_string()),
        },
    });
    for message in pending.into_iter().chain(ended) {
        if frames.send(message).await.is_err() {
            return;
        }
    }
}

async fn reserved(slot: &mut Option<Slot>) -> Result<OwnedPermit<Message>, SendError<()>> {
    match slot {
        Some(reserving) => reserving.await,
        None => future::pending().await,
    }
}

#[cfg(test)]
mod tests {
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

    fn wire(message: Message) -> Value {
        serde_json::to_value(message).unwrap()
    }

    #[tokio::test]
    async fn a_client_that_falls_behind_holds_herdr_back_without_losing_frames() {
        let dir = tempfile::tempdir().unwrap();
        let stream = open(dir.path(), "herdr", FRAMES, false);
        let (_controls, receiver) = mpsc::unbounded_channel();
        let (frames, mut queued) = mpsc::channel(4);
        tokio::spawn(watch(stream, "w1".to_owned(), receiver, frames));

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
            tokio::spawn(watch(stream, name.to_owned(), receiver, frames.clone()));
        }
        tokio::time::sleep(Duration::from_millis(300)).await;

        let (_busy, receiver) = mpsc::unbounded_channel();
        let stream = open(dir.path(), "busy", 3, true);
        tokio::spawn(watch(stream, "busy".to_owned(), receiver, frames));

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
}
