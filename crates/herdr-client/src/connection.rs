use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, oneshot};

use crate::Error;
use crate::wire::{self, CREDIT, DATA};
use message::{Greeting, Hello, Opened, Request, answer};
use mux::{Mux, Pending, read};

mod message;
mod mux;

const GENERATION: u32 = 1;
const PATIENCE: Duration = Duration::from_secs(30);

pub struct Connection {
    version: String,
    features: Vec<String>,
    outgoing: mpsc::UnboundedSender<Vec<u8>>,
    mux: Arc<Mux>,
    ids: AtomicU64,
    child: Mutex<Option<Child>>,
}

pub(crate) enum Chunk {
    Data(Vec<u8>),
    Closed(Option<String>),
}

pub(crate) struct Channel {
    pub(crate) id: u32,
    pub(crate) chunks: mpsc::UnboundedReceiver<Chunk>,
    outgoing: mpsc::UnboundedSender<Vec<u8>>,
    mux: Arc<Mux>,
}

impl Connection {
    pub async fn open(
        reader: impl AsyncRead + Send + Unpin + 'static,
        mut writer: impl AsyncWrite + Send + Unpin + 'static,
        child: Option<Child>,
    ) -> Result<Self, Error> {
        let mut reader = BufReader::new(reader);
        let mut hello = serde_json::to_vec(&Request {
            jsonrpc: "2.0",
            id: 0,
            method: "hello",
            params: Hello {
                generation: GENERATION,
                framed: true,
            },
        })?;
        hello.push(b'\n');
        writer.write_all(&hello).await?;
        writer.flush().await?;

        let mut line = String::new();
        let greeted = tokio::time::timeout(PATIENCE, reader.read_line(&mut line))
            .await
            .map_err(|_| Error::Timeout)?;
        if greeted? == 0 {
            return Err(Error::Closed);
        }
        let greeting: Greeting = serde_json::from_value(answer(&line)?.1?)?;

        let (outgoing, queued) = mpsc::unbounded_channel();
        let mux = Arc::new(Mux::new(outgoing.clone()));
        tokio::spawn(write(writer, queued));
        tokio::spawn(read(reader, Arc::clone(&mux)));

        Ok(Self {
            version: greeting.version,
            features: greeting.features,
            outgoing,
            mux,
            ids: AtomicU64::new(1),
            child: Mutex::new(child),
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn supports(&self, feature: &str) -> bool {
        self.features.iter().any(|offered| offered == feature)
    }

    pub fn is_closed(&self) -> bool {
        self.mux.closed.load(Ordering::SeqCst) || self.outgoing.is_closed()
    }

    pub async fn call<R: DeserializeOwned>(
        &self,
        method: &str,
        params: impl Serialize,
    ) -> Result<R, Error> {
        let value = self.request(method, params, None).await?;

        Ok(serde_json::from_value(value)?)
    }

    pub(crate) async fn open_channel<R: DeserializeOwned>(
        &self,
        method: &str,
        params: impl Serialize,
    ) -> Result<(R, Channel), Error> {
        let (stream, chunks) = mpsc::unbounded_channel();
        let value = self.request(method, params, Some(stream)).await?;
        let opened: Opened = serde_json::from_value(value.clone())?;
        let channel = Channel {
            id: opened.channel,
            chunks,
            outgoing: self.outgoing.clone(),
            mux: Arc::clone(&self.mux),
        };

        Ok((serde_json::from_value(value)?, channel))
    }

    async fn request(
        &self,
        method: &str,
        params: impl Serialize,
        stream: Option<mpsc::UnboundedSender<Chunk>>,
    ) -> Result<Value, Error> {
        let id = self.ids.fetch_add(1, Ordering::SeqCst);
        let message = serde_json::to_vec(&Request {
            jsonrpc: "2.0",
            id,
            method,
            params,
        })?;
        let (reply, answered) = oneshot::channel();
        lock(&self.mux.pending).insert(id, Pending { reply, stream });
        if self.mux.closed.load(Ordering::SeqCst)
            || self.outgoing.send(wire::encode(0, DATA, &message)).is_err()
        {
            lock(&self.mux.pending).remove(&id);
            return Err(Error::Closed);
        }

        let Ok(answer) = tokio::time::timeout(PATIENCE, answered).await else {
            lock(&self.mux.pending).remove(&id);
            self.end();
            return Err(Error::Timeout);
        };

        answer.map_err(|_| Error::Closed)?
    }

    fn end(&self) {
        self.mux.shut();
        if let Some(child) = lock(&self.child).as_mut()
            && let Err(error) = child.start_kill()
        {
            tracing::debug!(%error, "could not stop the Herdr connection");
        }
    }
}

impl Channel {
    pub(crate) fn send(&self, kind: u8, payload: &[u8]) -> Result<(), Error> {
        self.outgoing
            .send(wire::encode(self.id, kind, payload))
            .map_err(|_| Error::Closed)
    }

    pub(crate) fn credit(&self, bytes: usize) {
        let bytes = u32::try_from(bytes).unwrap_or(u32::MAX);
        let _ = self.send(CREDIT, &bytes.to_le_bytes());
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        self.mux.retire(self.id);
    }
}

async fn write(mut writer: impl AsyncWrite + Unpin, mut queued: mpsc::UnboundedReceiver<Vec<u8>>) {
    while let Some(frame) = queued.recv().await {
        if writer.write_all(&frame).await.is_err() || writer.flush().await.is_err() {
            return;
        }
    }
}

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
pub(crate) mod tests;
