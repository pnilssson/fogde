use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use herdr_client::{Error, Stream};
use protocol::rpc::{Failure, Message};
use protocol::terminals::{
    Display, EndedEvent, OutputEvent, Size, TerminalEnded, TerminalOutput, WatchTerminalParams,
};
use tokio::sync::mpsc;

use super::launch::{self, Launches};
use crate::rpc::{failure, notification};
use crate::session::Session;
use reattach::{Reattach, restarting, waiting};

const NORMAL: [&str; 2] = ["exited", "killed"];
const LOST: [&str; 5] = [
    "holder_lost",
    "stopped_by_restart",
    "server_stop",
    "host_shutdown",
    "signaled",
];
const PAUSE: Duration = Duration::from_secs(1);
const RESET: &[u8] = b"\x1bc";

pub enum Control {
    Input(Vec<u8>),
    Reply(Vec<u8>),
    Focus(bool),
    Resize(Size),
    Display(Display),
}

struct Watch<R> {
    id: String,
    token: u32,
    size: Size,
    display: Display,
    frames: mpsc::Sender<Message>,
    launches: Launches,
    herdr: R,
    pause: Duration,
    fresh: bool,
}

enum End {
    Gone,
    Failed(Error),
}

pub async fn open(session: &Session, params: WatchTerminalParams) -> Result<(), Failure> {
    let (controls, receiver) = mpsc::unbounded_channel();
    session
        .terminals
        .lock()
        .insert(params.id.clone(), controls.clone());

    let attached = async {
        let herdr = Arc::clone(session.herdr().await?);
        match herdr
            .attach(&params.id, params.size, params.display.clone())
            .await
        {
            Ok(stream) => Ok((herdr, Some(stream))),
            Err(error) if restarting(&error) => Ok((herdr, None)),
            Err(error) => Err(failure(error)),
        }
    }
    .await;
    let (herdr, stream) = match attached {
        Ok(attached) => attached,
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
        receiver,
        Watch {
            id: params.id,
            token: params.token,
            size: params.size,
            display: params.display,
            frames: session.frames.clone(),
            launches: session.terminals.launches.clone(),
            herdr,
            pause: PAUSE,
            fresh: false,
        },
    ));

    Ok(())
}

async fn watch<R: Reattach>(
    mut stream: Option<Stream>,
    mut controls: mpsc::UnboundedReceiver<Control>,
    mut watch: Watch<R>,
) {
    loop {
        let mut current = match stream.take() {
            Some(current) => current,
            None => match watch.reattach(&mut controls).await {
                Some(Ok(next)) => {
                    watch.fresh = true;
                    next
                }
                Some(Err(error)) => {
                    watch.end(error).await;
                    return;
                }
                None => return,
            },
        };
        let error = match watch.relay(&mut current, &mut controls).await {
            End::Gone => return,
            End::Failed(error) => error,
        };
        if !lost(&error) {
            watch.end(error).await;
            return;
        }

        tracing::info!(%error, id = %watch.id, "terminal lost, waiting for its new shell");
    }
}

impl<R: Reattach> Watch<R> {
    async fn relay(
        &mut self,
        stream: &mut Stream,
        controls: &mut mpsc::UnboundedReceiver<Control>,
    ) -> End {
        let frames = self.frames.clone();
        let mut pending: Option<Vec<u8>> = None;

        let error = loop {
            tokio::select! {
                permit = frames.reserve(), if pending.is_some() => {
                    let Ok(permit) = permit else {
                        return End::Gone;
                    };
                    if let Some(message) = pending.take().and_then(|data| self.output(&data)) {
                        permit.send(message);
                    }
                }
                output = stream.next(), if pending.is_none() => match output {
                    Ok(data) => {
                        if let Some(agent) = self.launches.take(&self.id)
                            && let Err(error) = stream.input(launch::command(agent).as_bytes())
                        {
                            break error;
                        }
                        pending = Some(data);
                    }
                    Err(error) => break error,
                },
                control = controls.recv() => {
                    let Some(control) = control else {
                        return End::Gone;
                    };
                    if let Err(error) = self.apply(stream, control) {
                        break error;
                    }
                }
            }
        };

        if let Some(message) = pending.and_then(|data| self.output(&data))
            && self.frames.send(message).await.is_err()
        {
            return End::Gone;
        }

        End::Failed(error)
    }

    fn apply(&mut self, stream: &Stream, control: Control) -> Result<(), Error> {
        match control {
            Control::Input(bytes) => stream.input(&bytes),
            Control::Reply(bytes) => stream.reply(&bytes),
            Control::Focus(focused) => stream.focus(focused),
            Control::Resize(size) => {
                self.size = size;
                stream.resize(size.cols, size.rows)
            }
            Control::Display(display) => {
                self.display = display.clone();
                stream.display(&facts(display))
            }
        }
    }

    async fn reattach(
        &mut self,
        controls: &mut mpsc::UnboundedReceiver<Control>,
    ) -> Option<Result<Stream, Error>> {
        loop {
            tokio::select! {
                () = tokio::time::sleep(self.pause) => {}
                control = controls.recv() => {
                    match control? {
                        Control::Resize(size) => self.size = size,
                        Control::Display(display) => self.display = display,
                        Control::Input(_) | Control::Reply(_) | Control::Focus(_) => {}
                    }
                    continue;
                }
            }

            match self
                .herdr
                .attach(&self.id, self.size, self.display.clone())
                .await
            {
                Ok(stream) => return Some(Ok(stream)),
                Err(error) if waiting(&error) => {}
                Err(error) => return Some(Err(error)),
            }
        }
    }

    async fn end(&self, error: Error) {
        tracing::info!(%error, id = %self.id, "terminal stream ended");
        let ended = notification::<EndedEvent>(TerminalEnded {
            id: self.id.clone(),
            error: match error {
                Error::Ended(reason) if NORMAL.contains(&reason.as_str()) => None,
                error => Some(error.to_string()),
            },
        });
        if let Some(message) = ended {
            let _ = self.frames.send(message).await;
        }
    }

    fn output(&mut self, data: &[u8]) -> Option<Message> {
        let data = if std::mem::take(&mut self.fresh) {
            [RESET, data].concat()
        } else {
            data.to_vec()
        };

        notification::<OutputEvent>(TerminalOutput {
            id: self.id.clone(),
            token: self.token,
            data: STANDARD.encode(data),
        })
    }
}

fn lost(error: &Error) -> bool {
    match error {
        Error::Ended(reason) => LOST.contains(&reason.as_str()),
        Error::Closed | Error::Io(_) | Error::Timeout => true,
        _ => false,
    }
}

pub(super) fn facts(display: Display) -> herdr_client::Display {
    herdr_client::Display {
        fg: display.fg,
        bg: display.bg,
        cursor: display.cursor,
    }
}

mod reattach;
#[cfg(test)]
mod tests;
