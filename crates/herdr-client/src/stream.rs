use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;

use crate::connection::{Channel, Chunk};
use crate::wire::{self, AUTOMATIC, DATA, FACTS, FOCUS, RESIZE};
use crate::{Connection, Error};

const HISTORY: usize = 2000;
const INPUT: usize = 64 * 1024;
const TAGGED: &str = "automatic_input";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Display {
    pub fg: String,
    pub bg: String,
    pub cursor: String,
}

pub struct Stream {
    channel: Channel,
    ended: Option<Error>,
    tagged: bool,
}

impl Connection {
    pub async fn attach(
        &self,
        terminal: &str,
        cols: u16,
        rows: u16,
        display: Display,
    ) -> Result<Stream, Error> {
        let (attached, channel): (Attached, _) = self
            .open_channel(
                "terminal.attach",
                Attach {
                    terminal,
                    cols,
                    rows,
                    history: HISTORY,
                    display,
                },
            )
            .await?;

        Ok(Stream {
            channel,
            ended: None,
            tagged: attached.automatic_input && self.supports(TAGGED),
        })
    }
}

impl Stream {
    pub async fn next(&mut self) -> Result<Vec<u8>, Error> {
        if let Some(error) = self.ended.take() {
            return Err(error);
        }

        let mut bytes = match self.channel.chunks.recv().await {
            Some(Chunk::Data(bytes)) => bytes,
            Some(Chunk::Closed(reason)) => return Err(ended(reason)),
            None => return Err(Error::Closed),
        };
        loop {
            match self.channel.chunks.try_recv() {
                Ok(Chunk::Data(more)) => bytes.extend(more),
                Ok(Chunk::Closed(reason)) => {
                    self.ended = Some(ended(reason));
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.ended = Some(Error::Closed);
                    break;
                }
            }
        }
        self.channel.credit(bytes.len());

        Ok(bytes)
    }

    pub fn input(&self, bytes: &[u8]) -> Result<(), Error> {
        self.send(DATA, bytes)
    }

    pub fn reply(&self, bytes: &[u8]) -> Result<(), Error> {
        self.send(if self.tagged { AUTOMATIC } else { DATA }, bytes)
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), Error> {
        self.channel.send(RESIZE, &wire::size(cols, rows))
    }

    pub fn display(&self, display: &Display) -> Result<(), Error> {
        self.channel.send(FACTS, &serde_json::to_vec(display)?)
    }

    pub fn focus(&self, focused: bool) -> Result<(), Error> {
        self.channel.send(FOCUS, &[u8::from(focused)])
    }

    fn send(&self, kind: u8, bytes: &[u8]) -> Result<(), Error> {
        for chunk in bytes.chunks(INPUT) {
            self.channel.send(kind, chunk)?;
        }

        Ok(())
    }
}

fn ended(reason: Option<String>) -> Error {
    reason.map_or(Error::Closed, Error::Ended)
}

#[derive(Serialize)]
struct Attach<'a> {
    terminal: &'a str,
    cols: u16,
    rows: u16,
    history: usize,
    display: Display,
}

#[derive(Deserialize)]
struct Attached {
    #[serde(default)]
    automatic_input: bool,
}

#[cfg(test)]
mod tests;
