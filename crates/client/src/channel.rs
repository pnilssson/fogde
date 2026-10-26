use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use protocol::Connection;
use protocol::rpc::Failure;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio::task::AbortHandle;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::client::Inner;
use crate::error::Error;
use crate::lock;

const OPEN: Duration = Duration::from_secs(30);

type Reply = oneshot::Sender<Result<Value, Error>>;

#[derive(Clone)]
pub(crate) struct Channel(Arc<Parts>);

struct Parts {
    outgoing: mpsc::UnboundedSender<String>,
    pending: Mutex<Option<HashMap<u64, Reply>>>,
    buffered: AtomicUsize,
    tasks: Mutex<Vec<AbortHandle>>,
}

#[derive(Deserialize)]
struct Incoming {
    id: Option<u64>,
    method: Option<String>,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    result: Value,
    error: Option<Failure>,
}

impl Channel {
    pub(crate) async fn open(
        connection: &Connection,
        generation: u64,
        client: Weak<Inner>,
    ) -> Result<Self, Error> {
        let url = format!("{}?token={}", connection.url, connection.token);
        let socket = match timeout(OPEN, connect_async(url)).await {
            Ok(Ok((socket, _))) => socket,
            Ok(Err(error)) => {
                tracing::debug!(%error, "could not open the host connection");
                return Err(Error::Unreachable);
            }
            Err(_) => return Err(Error::Timeout),
        };

        let (mut sink, mut stream) = socket.split();
        let (outgoing, mut queue) = mpsc::unbounded_channel::<String>();
        let channel = Self(Arc::new(Parts {
            outgoing,
            pending: Mutex::new(Some(HashMap::new())),
            buffered: AtomicUsize::new(0),
            tasks: Mutex::new(Vec::new()),
        }));

        let parts = Arc::downgrade(&channel.0);
        let writer = tokio::spawn(async move {
            while let Some(text) = queue.recv().await {
                let size = text.len();
                let sent = sink.send(Message::text(text)).await;
                if let Some(parts) = parts.upgrade() {
                    parts.buffered.fetch_sub(size, Ordering::Relaxed);
                }
                if sent.is_err() {
                    break;
                }
            }
        });

        let reader = tokio::spawn({
            let channel = channel.clone();
            async move {
                while let Some(Ok(message)) = stream.next().await {
                    let Message::Text(text) = message else {
                        continue;
                    };
                    if !channel.receive(&text, generation, &client) {
                        break;
                    }
                }

                channel.close();
                if let Some(inner) = client.upgrade() {
                    inner.dropped(generation);
                }
            }
        });

        lock(&channel.0.tasks).extend([writer.abort_handle(), reader.abort_handle()]);

        Ok(channel)
    }

    pub(crate) fn request(
        &self,
        id: u64,
        text: String,
    ) -> Result<oneshot::Receiver<Result<Value, Error>>, Error> {
        let (reply, answer) = oneshot::channel();
        lock(&self.0.pending)
            .as_mut()
            .ok_or(Error::Closed)?
            .insert(id, reply);

        let size = text.len();
        self.0.buffered.fetch_add(size, Ordering::Relaxed);
        if self.0.outgoing.send(text).is_err() {
            self.0.buffered.fetch_sub(size, Ordering::Relaxed);
            if let Some(pending) = lock(&self.0.pending).as_mut() {
                pending.remove(&id);
            }
            return Err(Error::Closed);
        }

        Ok(answer)
    }

    pub(crate) fn buffered(&self) -> usize {
        self.0.buffered.load(Ordering::Relaxed)
    }

    pub(crate) fn close(&self) {
        let pending = lock(&self.0.pending).take();
        for reply in pending.into_iter().flat_map(HashMap::into_values) {
            let _ = reply.send(Err(Error::Lost));
        }

        for task in lock(&self.0.tasks).drain(..) {
            task.abort();
        }
    }

    fn receive(&self, text: &str, generation: u64, client: &Weak<Inner>) -> bool {
        let incoming = match serde_json::from_str::<Incoming>(text) {
            Ok(incoming) => incoming,
            Err(error) => {
                tracing::warn!(%error, "could not read a host message");
                return true;
            }
        };

        if let Some(method) = incoming.method {
            let Some(inner) = client.upgrade() else {
                return false;
            };
            inner.deliver(&method, &incoming.params, generation);
            return true;
        }

        let Some(id) = incoming.id else {
            return true;
        };

        let reply = lock(&self.0.pending)
            .as_mut()
            .and_then(|pending| pending.remove(&id));
        if let Some(reply) = reply {
            let outcome = match incoming.error {
                Some(failure) => Err(Error::Rpc(failure)),
                None => Ok(incoming.result),
            };
            let _ = reply.send(outcome);
        }

        true
    }
}
