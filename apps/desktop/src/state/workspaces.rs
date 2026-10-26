mod actions;

use std::collections::{HashMap, HashSet};

use gpui_kit::{Context, Entity, EventEmitter, SharedString, Subscription, Task};
use protocol::terminals::{FailedEvent, StateEvent, Terminal, TerminalState};
use protocol::workspaces::{
    ChangedEvent, ListWorkspaces, MoveTerminal, MoveTerminalParams, PlaceTerminal,
    PlaceTerminalParams, Workspace,
};

use crate::notice;
use crate::runtime;
use crate::state::cache::{Applied, Cache};
use crate::state::listen::drain;
use crate::state::machines::{Change, Machines};

pub enum Activity {
    State {
        machine: SharedString,
        state: TerminalState,
        applied: Applied,
    },
}

struct Watch {
    _tasks: Vec<Task<()>>,
}

pub struct Workspaces {
    machines: Entity<Machines>,
    caches: HashMap<SharedString, Cache>,
    watches: HashMap<SharedString, Watch>,
    loads: HashMap<SharedString, Task<()>>,
    placing: HashSet<(SharedString, String)>,
    closing: HashSet<(SharedString, String)>,
    removing: HashSet<(SharedString, String)>,
    _machines: Subscription,
}

impl EventEmitter<Activity> for Workspaces {}

impl Workspaces {
    pub fn new(machines: Entity<Machines>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe(&machines, |this, _, change, cx| match change {
            Change::Listed => this.sync(cx),
            Change::Connected(id) => this.reload(id, cx),
        });
        let mut workspaces = Self {
            machines,
            caches: HashMap::new(),
            watches: HashMap::new(),
            loads: HashMap::new(),
            placing: HashSet::new(),
            closing: HashSet::new(),
            removing: HashSet::new(),
            _machines: subscription,
        };
        workspaces.sync(cx);
        workspaces
    }

    pub fn cache(&self, machine: &str) -> Option<&Cache> {
        self.caches.get(machine)
    }

    pub fn list(&self, machine: &str) -> &[Workspace] {
        self.cache(machine)
            .and_then(Cache::list)
            .unwrap_or_default()
    }

    pub fn workspace(&self, machine: &str, id: &str) -> Option<&Workspace> {
        self.list(machine)
            .iter()
            .find(|workspace| workspace.id == id)
    }

    pub fn terminal(&self, machine: &str, id: &str) -> Option<(&Workspace, &Terminal)> {
        self.list(machine).iter().find_map(|workspace| {
            workspace
                .terminals
                .iter()
                .find(|terminal| terminal.id == id)
                .map(|terminal| (workspace, terminal))
        })
    }

    pub fn placing(&self, machine: &str, terminal: &str) -> bool {
        self.placing
            .iter()
            .any(|(known, id)| known == machine && id == terminal)
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let machines = self.machines.read(cx);
        let connected: Vec<_> = machines
            .connected()
            .map(|machine| (machine.id.clone(), machine.client.clone()))
            .collect();

        self.caches
            .retain(|id, _| connected.iter().any(|(known, _)| known == id));
        self.watches
            .retain(|id, _| connected.iter().any(|(known, _)| known == id));
        for (id, client) in connected {
            if self.watches.contains_key(&id) {
                continue;
            }

            let changed = {
                let id = id.clone();
                drain::<ChangedEvent, _>(&client, cx, move |this, (), cx| {
                    this.reload(&id, cx);
                    true
                })
            };
            let states = {
                let id = id.clone();
                drain::<StateEvent, _>(&client, cx, move |this, state, cx| {
                    this.state(&id, state, cx);
                    true
                })
            };
            let failures = {
                let id = id.clone();
                drain::<FailedEvent, _>(&client, cx, move |this, failed, cx| {
                    this.reload(&id, cx);
                    notice::error(
                        format!("Could not start {}", failed.agent),
                        &failed.error,
                        cx,
                    );
                    true
                })
            };
            self.watches.insert(
                id.clone(),
                Watch {
                    _tasks: vec![changed, states, failures],
                },
            );
            self.caches.entry(id.clone()).or_default();
            self.reload(&id, cx);
        }
        cx.notify();
    }

    pub fn reload(&mut self, machine: &str, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        let id = SharedString::from(machine.to_owned());
        let ticket = self.caches.entry(id.clone()).or_default().begin();
        let request = runtime::call::<ListWorkspaces>(&client, (), cx);
        let key = id.clone();
        let task = cx.spawn(async move |this, cx| {
            let result = request.await.map_err(|error| error.to_string());
            this.update(cx, |this, cx| {
                if let Some(cache) = this.caches.get_mut(&key)
                    && cache.finish(ticket, result)
                {
                    cx.notify();
                }
            })
            .ok();
        });
        self.loads.insert(id, task);
        cx.notify();
    }

    fn state(&mut self, machine: &SharedString, state: TerminalState, cx: &mut Context<Self>) {
        let applied = self
            .caches
            .get_mut(machine)
            .and_then(|cache| cache.apply(&state));
        let Some(applied) = applied else {
            self.reload(machine, cx);
            return;
        };

        if applied.moved {
            self.reload(machine, cx);
        }
        cx.emit(Activity::State {
            machine: machine.clone(),
            state,
            applied,
        });
    }

    pub fn edit(&mut self, machine: &str, change: impl FnOnce(&mut Cache), cx: &mut Context<Self>) {
        if let Some(cache) = self.caches.get_mut(machine) {
            change(cache);
        }
        self.reload(machine, cx);
    }

    pub fn add(
        &mut self,
        machine: &str,
        workspace: &str,
        terminal: Terminal,
        cx: &mut Context<Self>,
    ) {
        self.edit(machine, |cache| cache.add(workspace, terminal), cx);
    }

    pub fn remove(&mut self, machine: &str, terminal: &str, cx: &mut Context<Self>) {
        self.edit(machine, |cache| cache.remove(terminal), cx);
    }

    pub fn place(&mut self, machine: &str, params: PlaceTerminalParams, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        let key = (
            SharedString::from(machine.to_owned()),
            params.terminal.clone(),
        );
        self.placing.insert(key.clone());
        if let Some(cache) = self.caches.get_mut(machine) {
            cache.place(&params);
        }
        cx.notify();

        let request = runtime::call::<PlaceTerminal>(&client, params, cx);
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                this.placing.remove(&key);
                if let Err(error) = result {
                    notice::error("Could not move the terminal", &error, cx);
                }
                this.reload(&key.0, cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn arrange(&mut self, machine: &str, params: MoveTerminalParams, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        if let Some(cache) = self.caches.get_mut(machine) {
            cache.arrange(&params);
        }
        cx.notify();

        let id = SharedString::from(machine.to_owned());
        let request = runtime::call::<MoveTerminal>(&client, params, cx);
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    notice::error("Could not move the terminal", &error, cx);
                }
                this.reload(&id, cx);
            })
            .ok();
        })
        .detach();
    }
}
