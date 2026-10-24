use std::future;
use std::pin::Pin;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use herdr_client::{Client, Key};
use protocol::rpc::{Failure, Message, SERVER_ERROR};
use protocol::terminals::{
    EndedEvent, FrameEvent, Scroll, ScrollTerminalParams, Size, TerminalEnded, TerminalFrame,
    WatchTerminalParams,
};
use tokio::sync::mpsc::{self, OwnedPermit, error::SendError};

use crate::rpc::{cancelled, failure, notification};
use crate::session::Session;

mod scroll;

pub enum Control {
    Input(Vec<u8>),
    Key(Key),
    Resize(Size),
    Scroll(ScrollTerminalParams),
}

type Slot = Pin<Box<dyn Future<Output = Result<OwnedPermit<Message>, SendError<()>>> + Send>>;

pub async fn open(session: &Session, params: WatchTerminalParams) -> Result<(), Failure> {
    let herdr = session.herdr().await?;
    let pane = herdr
        .client()
        .panes()
        .await
        .map_err(failure)?
        .into_iter()
        .find(|pane| pane.workspace == params.id)
        .ok_or_else(|| Failure::new(SERVER_ERROR, "the terminal is gone"))?;
    let stream = herdr_client::Terminal::open(
        herdr.program(),
        &herdr.socket(),
        &pane.id,
        params.size.cols,
        params.size.rows,
    )
    .map_err(failure)?;

    let (controls, receiver) = mpsc::unbounded_channel();
    let mut watches = session.terminals.lock();
    if session.is_closed() {
        return Err(cancelled());
    }
    watches.insert(params.id.clone(), controls);
    drop(watches);
    let keys = press(herdr.client(), pane.id.clone());
    let watching = tokio::spawn(watch(
        stream,
        keys,
        params.id.clone(),
        receiver,
        session.frames.clone(),
    ));
    tokio::spawn(scroll::follow(
        herdr.client(),
        pane.id,
        params.id,
        session.outgoing.clone(),
        watching,
    ));

    Ok(())
}

fn press(client: Client, pane: String) -> mpsc::UnboundedSender<Key> {
    let (keys, mut pressed) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(key) = pressed.recv().await {
            if let Err(error) = client.send_keys(&pane, &[key]).await {
                tracing::warn!(%error, %pane, ?key, "could not send a key to the terminal");
            }
        }
    });

    keys
}

async fn watch(
    mut stream: herdr_client::Terminal,
    keys: mpsc::UnboundedSender<Key>,
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
                    Some(Control::Key(key)) => {
                        let _ = keys.send(key);
                        Ok(())
                    }
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
mod tests;
