use serde::Serialize;
use serde::de::IgnoredAny;
use tokio::sync::mpsc::error::TryRecvError;

use crate::connection::{Channel, Chunk};
use crate::wire::{self, DATA, RESIZE};
use crate::{Connection, Error};

const HISTORY: usize = 2000;
const INPUT: usize = 64 * 1024;

pub struct Stream {
    channel: Channel,
    ended: Option<Error>,
}

impl Connection {
    pub async fn attach(&self, terminal: &str, cols: u16, rows: u16) -> Result<Stream, Error> {
        let (IgnoredAny, channel) = self
            .open_channel(
                "terminal.attach",
                Attach {
                    terminal,
                    cols,
                    rows,
                    history: HISTORY,
                },
            )
            .await?;

        Ok(Stream {
            channel,
            ended: None,
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
        for chunk in bytes.chunks(INPUT) {
            self.channel.send(DATA, chunk)?;
        }

        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), Error> {
        self.channel.send(RESIZE, &wire::size(cols, rows))
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
}
