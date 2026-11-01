use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use herdr_client::{Error, Stream};
use protocol::rpc::{Failure, Message};
use protocol::terminals::{
    EndedEvent, OutputEvent, Size, TerminalEnded, TerminalOutput, WatchTerminalParams,
};
use tokio::sync::mpsc;

use super::launch::{self, Launches};
use crate::rpc::{failure, notification};
use crate::session::Session;

const NORMAL: [&str; 2] = ["exited", "killed"];

pub enum Control {
    Input(Vec<u8>),
    Resize(Size),
}

pub async fn open(session: &Session, params: WatchTerminalParams) -> Result<(), Failure> {
    let (controls, receiver) = mpsc::unbounded_channel();
    session
        .terminals
        .lock()
        .insert(params.id.clone(), controls.clone());

    let attached = async {
        session
            .herdr()
            .await?
            .client()
            .await
            .map_err(failure)?
            .attach(&params.id, params.size.cols, params.size.rows)
            .await
            .map_err(failure)
    }
    .await;
    let stream = match attached {
        Ok(stream) => stream,
        Err(failure) => {
            let mut watches = session.terminals.lock();
            if watches
                .get(&params.id)
                .is_some_and(|current| current.same_channel(&controls))
            {
                watches.remove(&params.id);
            }
            return Err(failure);
        }
    };
    if session.is_closed() {
        return Err(failure(Error::Closed));
    }

    tokio::spawn(watch(
        stream,
        params.id,
        params.token,
        receiver,
        session.frames.clone(),
        session.terminals.launches.clone(),
    ));

    Ok(())
}

async fn watch(
    mut stream: Stream,
    id: String,
    token: u32,
    mut controls: mpsc::UnboundedReceiver<Control>,
    frames: mpsc::Sender<Message>,
    launches: Launches,
) {
    let mut pending: Option<Vec<u8>> = None;

    let error = loop {
        tokio::select! {
            permit = frames.reserve(), if pending.is_some() => {
                let Ok(permit) = permit else {
                    return;
                };
                if let Some(message) = pending.take().and_then(|data| output(&id, token, &data)) {
                    permit.send(message);
                }
            }
            output = stream.next(), if pending.is_none() => match output {
                Ok(data) => {
                    if let Some(agent) = launches.take(&id)
                        && let Err(error) = stream.input(launch::command(agent).as_bytes())
                    {
                        break error;
                    }
                    pending = Some(data);
                }
                Err(error) => break error,
            },
            control = controls.recv() => {
                let sent = match control {
                    Some(Control::Input(bytes)) => stream.input(&bytes),
                    Some(Control::Resize(size)) => stream.resize(size.cols, size.rows),
                    None => return,
                };
                if let Err(error) = sent {
                    break error;
                }
            }
        }
    };

    tracing::info!(%error, %id, "terminal stream ended");
    let drawn = pending.and_then(|data| output(&id, token, &data));
    let ended = notification::<EndedEvent>(TerminalEnded {
        id,
        error: match error {
            Error::Ended(reason) if NORMAL.contains(&reason.as_str()) => None,
            error => Some(error.to_string()),
        },
    });
    for message in drawn.into_iter().chain(ended) {
        if frames.send(message).await.is_err() {
            return;
        }
    }
}

fn output(id: &str, token: u32, data: &[u8]) -> Option<Message> {
    notification::<OutputEvent>(TerminalOutput {
        id: id.to_owned(),
        token,
        data: STANDARD.encode(data),
    })
}

#[cfg(test)]
mod tests;
