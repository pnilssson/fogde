use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use protocol::{Connection, Event, Method};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, watch};

use crate::channel::Channel;
use crate::error::Error;
use crate::keepalive::{self, Keepalive};
use crate::listener::Listener;
use crate::lock;

type Address = Box<dyn Fn() -> BoxFuture<'static, Result<Connection, Error>> + Send + Sync>;
type Opening = Shared<BoxFuture<'static, Result<Channel, Error>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Connecting,
    Connected,
    Disconnected { error: String, retryable: bool },
}

#[derive(Clone)]
pub struct Client(pub(crate) Arc<Inner>);

pub(crate) struct Inner {
    address: Address,
    link: Mutex<Link>,
    next: AtomicU64,
    state: watch::Sender<State>,
    listeners: Mutex<HashMap<&'static str, Vec<mpsc::UnboundedSender<Value>>>>,
}

struct Link {
    generation: u64,
    stage: Stage,
}

enum Stage {
    Idle,
    Opening(Opening, oneshot::Sender<()>),
    Open(Channel),
}

#[derive(Serialize)]
struct Call<'a, P> {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: &'a P,
}

impl Client {
    pub fn new<F, A>(address: F) -> Self
    where
        F: Fn() -> A + Send + Sync + 'static,
        A: Future<Output = Result<Connection, Error>> + Send + 'static,
    {
        let client = Self(Arc::new(Inner {
            address: Box::new(move || address().boxed()),
            link: Mutex::new(Link {
                generation: 0,
                stage: Stage::Idle,
            }),
            next: AtomicU64::new(0),
            state: watch::Sender::new(State::Idle),
            listeners: Mutex::new(HashMap::new()),
        }));
        tokio::spawn(keepalive::run(
            keepalive::Handle(Arc::downgrade(&client.0)),
            client.watch(),
            Keepalive::default(),
        ));

        client
    }

    pub fn local(connection: Connection) -> Self {
        Self::new(move || std::future::ready(Ok(connection.clone())))
    }

    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn state(&self) -> State {
        self.0.state.borrow().clone()
    }

    pub fn watch(&self) -> watch::Receiver<State> {
        self.0.state.subscribe()
    }

    pub fn listen<E: Event>(&self) -> Listener<E> {
        let (sender, receiver) = mpsc::unbounded_channel();
        lock(&self.0.listeners)
            .entry(E::NAME)
            .or_default()
            .push(sender);

        Listener::new(receiver)
    }

    pub async fn call<M: Method>(&self, params: M::Params) -> Result<M::Result, Error> {
        let channel = self.connect().await?;
        let id = self.0.next.fetch_add(1, Ordering::Relaxed) + 1;
        let call = Call {
            jsonrpc: "2.0",
            id,
            method: M::NAME,
            params: &params,
        };
        let text =
            serde_json::to_string(&call).map_err(|error| Error::Invalid(error.to_string()))?;

        let reply = channel
            .request(id, text)?
            .await
            .map_err(|_| Error::Lost)??;

        serde_json::from_value(reply).map_err(|error| Error::Invalid(error.to_string()))
    }

    pub fn reset(&self) {
        self.close(State::Idle);
    }

    pub fn lose(&self, error: &str) {
        self.close(State::Disconnected {
            error: error.to_owned(),
            retryable: true,
        });
    }

    pub async fn reconnect(&self) -> Result<(), Error> {
        self.reset();
        self.connect().await.map(|_| ())
    }

    pub fn buffered(&self) -> usize {
        match &lock(&self.0.link).stage {
            Stage::Open(channel) => channel.buffered(),
            Stage::Idle | Stage::Opening(..) => 0,
        }
    }

    async fn connect(&self) -> Result<Channel, Error> {
        let opening = {
            let mut link = lock(&self.0.link);
            match &link.stage {
                Stage::Open(channel) => return Ok(channel.clone()),
                Stage::Opening(opening, _) => opening.clone(),
                Stage::Idle => {
                    let (cancel, cancelled) = oneshot::channel();
                    let task = tokio::spawn(open(self.clone(), link.generation));
                    let opening = async move {
                        tokio::select! {
                            result = task => result.unwrap_or(Err(Error::Replaced)),
                            Ok(()) = cancelled => Err(Error::Replaced),
                        }
                    }
                    .boxed()
                    .shared();
                    link.stage = Stage::Opening(opening.clone(), cancel);
                    self.0.update(State::Connecting);
                    opening
                }
            }
        };

        opening.await
    }

    fn close(&self, state: State) {
        let previous = {
            let mut link = lock(&self.0.link);
            link.generation += 1;
            std::mem::replace(&mut link.stage, Stage::Idle)
        };

        match previous {
            Stage::Open(channel) => channel.close(),
            Stage::Opening(_, cancel) => {
                let _ = cancel.send(());
            }
            Stage::Idle => {}
        }

        self.0.update(state);
    }
}

async fn open(client: Client, generation: u64) -> Result<Channel, Error> {
    let result = async {
        let connection = (client.0.address)().await?;
        if lock(&client.0.link).generation != generation {
            return Err(Error::Replaced);
        }

        Channel::open(&connection, generation, Arc::downgrade(&client.0)).await
    }
    .await;

    let mut link = lock(&client.0.link);
    if link.generation != generation {
        if let Ok(channel) = &result {
            channel.close();
        }
        return Err(Error::Replaced);
    }

    match &result {
        Ok(channel) => {
            link.stage = Stage::Open(channel.clone());
            client.0.update(State::Connected);
        }
        Err(error) => {
            link.stage = Stage::Idle;
            client.0.update(State::Disconnected {
                error: error.to_string(),
                retryable: error.retryable(),
            });
        }
    }

    result
}

impl Inner {
    fn update(&self, state: State) {
        self.state.send_if_modified(|current| {
            if *current == state {
                return false;
            }

            *current = state;
            true
        });
    }

    pub(crate) fn dropped(&self, generation: u64) {
        let mut link = lock(&self.link);
        if link.generation != generation || !matches!(link.stage, Stage::Open(_)) {
            return;
        }

        link.stage = Stage::Idle;
        self.update(State::Disconnected {
            error: Error::Lost.to_string(),
            retryable: true,
        });
    }

    pub(crate) fn deliver(&self, method: &str, params: &Value, generation: u64) {
        if lock(&self.link).generation != generation {
            return;
        }

        let senders = lock(&self.listeners).get_mut(method).map(|senders| {
            senders.retain(|sender| !sender.is_closed());
            senders.clone()
        });

        for sender in senders.into_iter().flatten() {
            let _ = sender.send(params.clone());
        }
    }
}
