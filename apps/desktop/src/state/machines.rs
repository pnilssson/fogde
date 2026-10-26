use std::collections::HashMap;

use client::{Client, State};
use gpui_kit::{Context, EventEmitter, SharedString, Task};
use protocol::Connection;
use protocol::machines::{ListMachines, Machine as Entry, MachineId};

use crate::notice;
use crate::runtime;

pub const LOCAL: &str = "local";

#[derive(Clone)]
pub struct Machine {
    pub id: SharedString,
    pub name: SharedString,
    pub target: Option<SharedString>,
    pub connected: bool,
    pub client: Client,
}

impl Machine {
    pub fn local(&self) -> bool {
        self.id == LOCAL
    }

    pub fn key(&self) -> MachineId {
        if self.local() {
            MachineId::Computer
        } else {
            MachineId::Server {
                id: self.id.to_string(),
            }
        }
    }
}

pub enum Change {
    Connected(SharedString),
    Listed,
}

pub struct Machines {
    own: Client,
    connection: Connection,
    list: Option<Vec<Machine>>,
    states: HashMap<SharedString, State>,
    watches: HashMap<SharedString, Task<()>>,
    loading: Option<Task<()>>,
    failure: Option<SharedString>,
}

impl EventEmitter<Change> for Machines {}

impl Machines {
    pub fn new(own: Client, connection: Connection, cx: &mut Context<Self>) -> Self {
        let mut machines = Self {
            own,
            connection,
            list: None,
            states: HashMap::new(),
            watches: HashMap::new(),
            loading: None,
            failure: None,
        };
        machines.load(cx);
        machines
    }

    pub fn own(&self) -> &Client {
        &self.own
    }

    pub fn failure(&self) -> Option<&SharedString> {
        self.failure.as_ref()
    }

    pub fn loaded(&self) -> bool {
        self.list.is_some()
    }

    pub fn all(&self) -> &[Machine] {
        self.list.as_deref().unwrap_or_default()
    }

    pub fn connected(&self) -> impl Iterator<Item = &Machine> {
        self.all().iter().filter(|machine| machine.connected)
    }

    pub fn get(&self, id: &str) -> Option<&Machine> {
        self.all().iter().find(|machine| machine.id == id)
    }

    pub fn state(&self, id: &str) -> State {
        self.states.get(id).cloned().unwrap_or(State::Idle)
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        self.failure = None;
        let request = runtime::call::<ListMachines>(&self.own, (), cx);
        self.loading = Some(cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| match result {
                Ok(entries) => {
                    this.failure = None;
                    this.replace(&entries, cx);
                }
                Err(error) if this.list.is_none() => {
                    this.failure = Some(error.to_string().into());
                    cx.notify();
                }
                Err(error) => notice::error("Could not load your machines", &error, cx),
            })
            .ok();
        }));
    }

    pub fn retry(&self, cx: &mut Context<Self>) {
        for machine in self.connected() {
            if let State::Disconnected {
                retryable: true, ..
            } = self.state(&machine.id)
            {
                let client = machine.client.clone();
                runtime::spawn(cx, async move { client.reconnect().await }).detach();
            }
        }
    }

    pub fn reconnect(&self, id: &str, cx: &mut Context<Self>) {
        if let Some(machine) = self.get(id) {
            let client = machine.client.clone();
            runtime::spawn(cx, async move { client.reconnect().await }).detach();
        }
    }

    fn replace(&mut self, entries: &[Entry], cx: &mut Context<Self>) {
        let handle = runtime::handle(cx);
        let _guard = handle.enter();

        let mut known: HashMap<SharedString, Client> = self
            .all()
            .iter()
            .filter(|machine| !machine.local())
            .map(|machine| (machine.id.clone(), machine.client.clone()))
            .collect();
        let list: Vec<Machine> = entries
            .iter()
            .map(|entry| match entry {
                Entry::Computer { name, connected } => Machine {
                    id: LOCAL.into(),
                    name: name.clone().into(),
                    target: None,
                    connected: *connected,
                    client: self.own.clone(),
                },
                Entry::Server {
                    id,
                    name,
                    target,
                    connected,
                } => {
                    let key = SharedString::from(id.clone());
                    let client = known.remove(&key).unwrap_or_else(|| {
                        Client::server(self.own.clone(), self.connection.clone(), id.clone())
                    });
                    Machine {
                        id: key,
                        name: name.clone().into(),
                        target: Some(target.clone().into()),
                        connected: *connected,
                        client,
                    }
                }
            })
            .collect();

        for (id, client) in known {
            client.reset();
            self.watches.remove(&id);
            self.states.remove(&id);
        }
        for machine in &list {
            if !machine.connected {
                if !machine.local() {
                    machine.client.reset();
                }
                continue;
            }
            if !self.watches.contains_key(&machine.id) {
                let task = Self::watch(machine, cx);
                self.watches.insert(machine.id.clone(), task);
            }
        }
        self.watches.retain(|id, _| {
            list.iter()
                .any(|machine| machine.id == *id && machine.connected)
        });

        self.list = Some(list);
        cx.emit(Change::Listed);
        cx.notify();
    }

    fn watch(machine: &Machine, cx: &mut Context<Self>) -> Task<()> {
        let id = machine.id.clone();
        let mut states = machine.client.watch();
        let client = machine.client.clone();
        runtime::spawn(cx, async move {
            client.call::<protocol::health::Ping>(()).await
        })
        .detach();

        cx.spawn(async move |this, cx| {
            loop {
                let state = states.borrow_and_update().clone();
                let alive = this
                    .update(cx, |this, cx| {
                        let before = this.states.insert(id.clone(), state.clone());
                        if state == State::Connected && before != Some(State::Connected) {
                            cx.emit(Change::Connected(id.clone()));
                        }
                        cx.notify();
                    })
                    .is_ok();
                if !alive || states.changed().await.is_err() {
                    break;
                }
            }
        })
    }
}
