#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use client::{Client, State};
use futures_util::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use protocol::terminals::{Size, UnwatchTerminal, WatchTerminal, WatchTerminalParams};
use tokio::sync::oneshot;

pub trait Calls: Clone + Send + Sync + 'static {
    fn watch(&self, id: String, size: Size) -> BoxFuture<'static, Result<(), client::Error>>;
    fn unwatch(&self, id: String) -> BoxFuture<'static, Result<(), client::Error>>;
    fn connected(&self) -> bool;
}

impl Calls for Client {
    fn watch(&self, id: String, size: Size) -> BoxFuture<'static, Result<(), client::Error>> {
        let client = self.clone();
        async move {
            client
                .call::<WatchTerminal>(WatchTerminalParams { id, size })
                .await
        }
        .boxed()
    }

    fn unwatch(&self, id: String) -> BoxFuture<'static, Result<(), client::Error>> {
        let client = self.clone();
        async move { client.call::<UnwatchTerminal>(id).await }.boxed()
    }

    fn connected(&self) -> bool {
        self.state() == State::Connected
    }
}

type Tail = Shared<BoxFuture<'static, ()>>;

#[derive(Clone)]
pub struct Queue(Arc<Mutex<Tail>>);

impl Default for Queue {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(async {}.boxed().shared())))
    }
}

impl Queue {
    pub fn for_machine(machine: &str) -> Self {
        static QUEUES: LazyLock<Mutex<HashMap<String, Queue>>> = LazyLock::new(Mutex::default);
        QUEUES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(machine.to_owned())
            .or_default()
            .clone()
    }

    fn push(&self, operation: impl Future<Output = ()> + Send + 'static) -> Tail {
        let mut tail = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let previous = tail.clone();
        let next = async move {
            previous.await;
            operation.await;
        }
        .boxed()
        .shared();
        *tail = next.clone();
        next
    }
}

pub struct Watch<C: Calls> {
    calls: C,
    queue: Queue,
    id: String,
    active: Arc<AtomicBool>,
    attached: Arc<AtomicBool>,
}

impl<C: Calls> Watch<C> {
    pub fn start(
        calls: C,
        queue: Queue,
        id: String,
        size: Size,
    ) -> (Self, oneshot::Receiver<Result<(), client::Error>>, Tail) {
        let active = Arc::new(AtomicBool::new(true));
        let attached = Arc::new(AtomicBool::new(false));
        let (done, result) = oneshot::channel();

        let operation = {
            let calls = calls.clone();
            let id = id.clone();
            let active = active.clone();
            let attached = attached.clone();
            async move {
                if !active.load(Ordering::SeqCst) {
                    return;
                }

                let outcome = calls.watch(id, size).await;
                if outcome.is_ok() {
                    attached.store(true, Ordering::SeqCst);
                }
                if active.load(Ordering::SeqCst) {
                    let _ = done.send(outcome);
                }
            }
        };
        let tail = queue.push(operation);

        let watch = Self {
            calls,
            queue,
            id,
            active,
            attached,
        };
        (watch, result, tail)
    }

    pub fn stop(&self, release: bool) -> Option<Tail> {
        if !self.active.swap(false, Ordering::SeqCst) {
            return None;
        }

        let calls = self.calls.clone();
        let id = self.id.clone();
        let attached = self.attached.clone();
        Some(self.queue.push(async move {
            if attached.load(Ordering::SeqCst) && release && calls.connected() {
                let _ = calls.unwatch(id).await;
            }
        }))
    }
}
