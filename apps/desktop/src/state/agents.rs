use std::collections::{HashMap, HashSet};

use gpui_kit::{Context, Entity, SharedString, Subscription, Task};
use protocol::agents::{Agent, AgentState, AgentStatus, ListAgents};

use crate::notice;
use crate::runtime;
use crate::state::machines::{Change, Machines};

pub struct Agents {
    machines: Entity<Machines>,
    lists: HashMap<SharedString, Vec<AgentStatus>>,
    loads: HashMap<SharedString, Task<()>>,
    failed: HashSet<SharedString>,
    _machines: Subscription,
}

impl Agents {
    pub fn new(machines: Entity<Machines>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe(&machines, |this, _, change, cx| match change {
            Change::Connected(id) => this.reload(id, cx),
            Change::Listed => {
                let known: Vec<SharedString> = this
                    .machines
                    .read(cx)
                    .all()
                    .iter()
                    .map(|machine| machine.id.clone())
                    .collect();
                this.lists.retain(|id, _| known.contains(id));
                this.failed.retain(|id| known.contains(id));
            }
        });
        let connected: Vec<SharedString> = machines
            .read(cx)
            .connected()
            .map(|machine| machine.id.clone())
            .collect();
        let mut agents = Self {
            machines,
            lists: HashMap::new(),
            loads: HashMap::new(),
            failed: HashSet::new(),
            _machines: subscription,
        };
        for id in connected {
            agents.reload(&id, cx);
        }
        agents
    }

    pub fn list(&self, machine: &str) -> Option<&[AgentStatus]> {
        self.lists.get(machine).map(Vec::as_slice)
    }

    pub fn signed_in(&self, machine: &str) -> Vec<Agent> {
        self.list(machine)
            .unwrap_or_default()
            .iter()
            .filter(|status| matches!(status.state, AgentState::SignedIn))
            .map(|status| status.agent)
            .collect()
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
        let request = runtime::call::<ListAgents>(&client, (), cx);
        let key = id.clone();
        self.loads.insert(
            id,
            cx.spawn(async move |this, cx| {
                let result = request.await;
                this.update(cx, |this, cx| {
                    match result {
                        Ok(list) => {
                            this.failed.remove(&key);
                            this.lists.insert(key, list);
                        }
                        Err(error) => {
                            let repeated = !this.failed.insert(key);
                            if !repeated && !error.retryable() {
                                notice::error("Could not load agents", &error, cx);
                            }
                        }
                    }
                    cx.notify();
                })
                .ok();
            }),
        );
    }
}
