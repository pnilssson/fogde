use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::AsyncRead;
use tokio::sync::{mpsc, oneshot};

use super::message::{Opened, answer, reason};
use super::{Chunk, lock};
use crate::Error;
use crate::wire::{self, CLOSE, DATA, Frame};

pub(super) struct Mux {
    pub(super) pending: Mutex<HashMap<u64, Pending>>,
    routes: Mutex<Routes>,
    outgoing: mpsc::UnboundedSender<Vec<u8>>,
    pub(super) closed: AtomicBool,
}

#[derive(Default)]
struct Routes {
    open: HashMap<u32, mpsc::UnboundedSender<Chunk>>,
    early: HashMap<u32, Vec<Chunk>>,
    done: HashSet<u32>,
}

pub(super) struct Pending {
    pub(super) reply: oneshot::Sender<Result<Value, Error>>,
    pub(super) stream: Option<mpsc::UnboundedSender<Chunk>>,
}

impl Mux {
    pub(super) fn new(outgoing: mpsc::UnboundedSender<Vec<u8>>) -> Self {
        Self {
            pending: Mutex::default(),
            routes: Mutex::default(),
            outgoing,
            closed: AtomicBool::new(false),
        }
    }

    fn answer(&self, payload: &[u8]) {
        let Ok(line) = std::str::from_utf8(payload) else {
            return;
        };
        let Ok((Some(id), result)) = answer(line) else {
            return;
        };
        let opened = result
            .as_ref()
            .ok()
            .and_then(|value| serde_json::from_value::<Opened>(value.clone()).ok())
            .map(|opened| opened.channel);
        let pending = lock(&self.pending).remove(&id);
        let stream = pending.as_ref().and_then(|pending| pending.stream.clone());
        match (opened, stream) {
            (Some(channel), Some(stream)) => self.open(channel, stream),
            (Some(channel), None) => self.retire(channel),
            _ => {}
        }

        let delivered = pending.is_some_and(|pending| pending.reply.send(result).is_ok());
        if !delivered && let Some(channel) = opened {
            self.retire(channel);
        }
    }

    fn open(&self, channel: u32, stream: mpsc::UnboundedSender<Chunk>) {
        let mut routes = lock(&self.routes);
        let mut ended = false;
        for chunk in routes.early.remove(&channel).unwrap_or_default() {
            ended |= matches!(chunk, Chunk::Closed(_));
            let _ = stream.send(chunk);
        }
        if ended {
            routes.done.insert(channel);
        } else {
            routes.open.insert(channel, stream);
        }
    }

    pub(super) fn retire(&self, channel: u32) {
        let mut routes = lock(&self.routes);
        routes.early.remove(&channel);
        let open = routes.open.remove(&channel).is_some();
        if routes.done.insert(channel) || open {
            let _ = self.outgoing.send(wire::encode(channel, CLOSE, b""));
        }
    }

    fn deliver(&self, channel: u32, chunk: Chunk) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }

        let ended = matches!(chunk, Chunk::Closed(_));
        let mut routes = lock(&self.routes);
        if let Some(stream) = routes.open.get(&channel) {
            let _ = stream.send(chunk);
            if ended {
                routes.open.remove(&channel);
                routes.done.insert(channel);
            }
        } else if !routes.done.contains(&channel) {
            routes.early.entry(channel).or_default().push(chunk);
        }
    }

    pub(super) fn shut(&self) {
        self.closed.store(true, Ordering::SeqCst);
        for (_, pending) in lock(&self.pending).drain() {
            let _ = pending.reply.send(Err(Error::Closed));
        }
        *lock(&self.routes) = Routes::default();
    }
}

pub(super) async fn read(mut reader: impl AsyncRead + Unpin, mux: Arc<Mux>) {
    loop {
        match wire::read(&mut reader).await {
            Ok(Some(Frame {
                channel: 0,
                kind: DATA,
                payload,
            })) => mux.answer(&payload),
            Ok(Some(Frame {
                channel,
                kind: DATA,
                payload,
            })) => mux.deliver(channel, Chunk::Data(payload)),
            Ok(Some(Frame {
                channel,
                kind: CLOSE,
                payload,
            })) => mux.deliver(channel, Chunk::Closed(reason(&payload))),
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(%error, "could not read from Herdr");
                break;
            }
        }
    }

    mux.shut();
}
