use std::future;
use std::pin::Pin;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use herdr_client::{Client, Frame, Key};
use protocol::rpc::{Failure, Message, SERVER_ERROR};
use protocol::terminals::{
    ClickTerminalParams, EndedEvent, FrameEvent, ScrollTerminalParams, Size, TerminalEnded,
    TerminalFrame, Viewport, WatchTerminalParams,
};
use tokio::sync::mpsc::{self, OwnedPermit, error::SendError};
use tokio::sync::oneshot;
use tokio::time;

use crate::rpc::{cancelled, failure, notification};
use crate::session::Session;

use super::{herdr, launch};

mod scroll;

const WAIT: Duration = Duration::from_millis(200);

pub enum Control {
    Input(Vec<u8>),
    Key(Key),
    Resize(Size),
    Scroll(ScrollTerminalParams),
    Click(ClickTerminalParams),
}

type Reserved = (
    Result<OwnedPermit<Message>, SendError<()>>,
    Option<Viewport>,
);
type Slot = Pin<Box<dyn Future<Output = Reserved> + Send>>;

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
    let attached = session.terminals.launches.holds(&params.id).then(|| {
        let (attached, ready) = oneshot::channel();
        let launches = session.terminals.launches.clone();
        let id = params.id.clone();
        let client = herdr.client();
        let outgoing = session.outgoing.clone();
        tokio::spawn(async move {
            if ready.await.is_err() {
                return;
            }

            if let Some(launching) = launches.take(&id) {
                launch::start(client, launching, outgoing);
            }
        });

        attached
    });
    let watching = tokio::spawn(watch(
        stream,
        herdr.client(),
        pane.id.clone(),
        params.id.clone(),
        receiver,
        session.frames.clone(),
        attached,
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
    client: Client,
    pane: String,
    id: String,
    mut controls: mpsc::UnboundedReceiver<Control>,
    frames: mpsc::Sender<Message>,
    mut attached: Option<oneshot::Sender<()>>,
) {
    let keys = press(client.clone(), pane.clone());
    let mut pending = None;
    let mut slot: Option<Slot> = None;

    let error = loop {
        tokio::select! {
            (permit, viewport) = reserved(&mut slot) => {
                slot = None;
                match (permit, pending.take().and_then(|frame| message(&id, frame, viewport))) {
                    (Ok(permit), Some(message)) => {
                        permit.send(message);
                    }
                    (Ok(_), None) => {}
                    (Err(_), _) => return,
                }
            }
            frame = stream.next(), if pending.is_none() => match frame {
                Ok(frame) => {
                    if let Some(attached) = attached.take() {
                        let _ = attached.send(());
                    }
                    pending = Some(frame);
                    slot = Some(Box::pin(reserve(client.clone(), pane.clone(), frames.clone())));
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
                        stream.scroll(herdr::scroll(scroll.direction), scroll.lines, scroll.column, scroll.row).await
                    }
                    Some(Control::Click(click)) => stream.click(click.column, click.row).await,
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
    let drawn = pending.and_then(|frame| message(&id, frame, None));
    let ended = notification::<EndedEvent>(TerminalEnded {
        id,
        error: match error {
            herdr_client::Error::Closed => None,
            error => Some(error.to_string()),
        },
    });
    for message in drawn.into_iter().chain(ended) {
        if frames.send(message).await.is_err() {
            return;
        }
    }
}

fn message(id: &str, frame: Frame, viewport: Option<Viewport>) -> Option<Message> {
    notification::<FrameEvent>(TerminalFrame {
        id: id.to_owned(),
        cols: frame.width,
        rows: frame.height,
        full: frame.full,
        data: STANDARD.encode(frame.bytes),
        viewport,
    })
}

async fn reserve(client: Client, pane: String, frames: mpsc::Sender<Message>) -> Reserved {
    let (scrollback, permit) = tokio::join!(
        time::timeout(WAIT, client.scrollback(&pane)),
        frames.reserve_owned()
    );
    let viewport = match scrollback {
        Ok(Ok(scrollback)) => scrollback.map(scroll::viewport),
        Ok(Err(error)) => {
            tracing::debug!(%error, %pane, "could not read where the terminal is scrolled");
            None
        }
        Err(_) => {
            tracing::debug!(%pane, "Herdr was slow to say where the terminal is scrolled");
            None
        }
    };

    (permit, viewport)
}

async fn reserved(slot: &mut Option<Slot>) -> Reserved {
    match slot {
        Some(reserving) => reserving.await,
        None => future::pending().await,
    }
}

#[cfg(test)]
mod tests;
