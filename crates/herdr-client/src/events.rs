use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::connection::{Channel, Chunk};
use crate::terminal::{Empty, Status, Terminal};
use crate::{Connection, Error};

pub struct Events {
    channel: Channel,
    buffer: Vec<u8>,
    seq: u64,
    terminals: BTreeMap<String, Terminal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    All,
    Turn(String),
    Terminal(String),
}

impl Connection {
    pub async fn events(&self) -> Result<Events, Error> {
        let (subscribed, channel): (Subscribed, _) =
            self.open_channel("events.subscribe", Empty {}).await?;

        Ok(Events {
            channel,
            buffer: Vec::new(),
            seq: subscribed.seq,
            terminals: by_address(subscribed.terminals),
        })
    }
}

impl Events {
    pub fn terminals(&self) -> &BTreeMap<String, Terminal> {
        &self.terminals
    }

    pub async fn changed(&mut self) -> Result<Change, Error> {
        loop {
            while let Some(end) = self.buffer.iter().position(|&byte| byte == b'\n') {
                let line: Vec<u8> = self.buffer.drain(..=end).collect();
                if let Some(change) = self.apply(&line)? {
                    return Ok(change);
                }
            }

            match self.channel.chunks.recv().await {
                Some(Chunk::Data(bytes)) => {
                    self.channel.credit(bytes.len());
                    self.buffer.extend(bytes);
                }
                Some(Chunk::Closed(reason)) => {
                    return Err(reason.map_or(Error::Closed, Error::Ended));
                }
                None => return Err(Error::Closed),
            }
        }
    }

    fn apply(&mut self, line: &[u8]) -> Result<Option<Change>, Error> {
        match serde_json::from_slice(line)? {
            Notification::Event(event) => Ok(self.event(event)),
            Notification::Missed(missed) => {
                self.seq = missed.seq;
                self.terminals = by_address(missed.terminals);

                Ok(Some(Change::All))
            }
            Notification::Heartbeat {} => Ok(None),
        }
    }

    fn event(&mut self, event: Event) -> Option<Change> {
        if event.seq <= self.seq {
            return None;
        }
        self.seq = event.seq;

        let address = event.terminal?;
        match event.kind.as_str() {
            "created" | "restarted" => {
                let terminal: Terminal = serde_json::from_value(event.data).ok()?;
                self.terminals.insert(address.clone(), terminal);
            }
            "finished" | "removed" => {
                self.terminals.remove(&address)?;
            }
            kind => {
                let terminal = self.terminals.get_mut(&address)?;
                match kind {
                    "lost" => {
                        terminal.pid = None;
                        terminal.foreground = None;
                    }
                    "status" => {
                        let reported: Reported = serde_json::from_value(event.data).ok()?;
                        terminal.status = reported.status;
                    }
                    "turn" => {
                        let ended: Ended = serde_json::from_value(event.data).ok()?;
                        terminal.turn = ended.turn;

                        return Some(Change::Turn(address));
                    }
                    "title" => {
                        let titled: Titled = serde_json::from_value(event.data).ok()?;
                        terminal.title = titled.title;
                    }
                    "labels" => {
                        let labelled: Labelled = serde_json::from_value(event.data).ok()?;
                        terminal.labels = labelled.labels;
                    }
                    "foreground" => {
                        terminal.foreground = serde_json::from_value(event.data).ok()?;
                    }
                    _ => return None,
                }
            }
        }

        Some(Change::Terminal(address))
    }
}

fn by_address(terminals: Vec<Terminal>) -> BTreeMap<String, Terminal> {
    terminals
        .into_iter()
        .map(|terminal| (terminal.address.clone(), terminal))
        .collect()
}

#[derive(Deserialize)]
struct Subscribed {
    seq: u64,
    terminals: Vec<Terminal>,
}

#[derive(Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "lowercase")]
enum Notification {
    Event(Event),
    Missed(Missed),
    Heartbeat {},
}

#[derive(Deserialize)]
struct Event {
    seq: u64,
    #[serde(rename = "type")]
    kind: String,
    terminal: Option<String>,
    #[serde(default)]
    data: Value,
}

#[derive(Deserialize)]
struct Missed {
    seq: u64,
    terminals: Vec<Terminal>,
}

#[derive(Deserialize)]
struct Reported {
    status: Option<Status>,
}

#[derive(Deserialize)]
struct Ended {
    turn: u64,
}

#[derive(Deserialize)]
struct Titled {
    title: String,
}

#[derive(Deserialize)]
struct Labelled {
    labels: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests;
