use std::collections::HashMap;

use agents::{Agent, SignIn};
use tokio::sync::oneshot;

#[derive(Default)]
pub struct SignIns {
    next: u64,
    flows: HashMap<Agent, Flow>,
}

enum Flow {
    Running {
        id: u64,
        cancel: oneshot::Sender<()>,
    },
    Waiting {
        id: u64,
        signin: Box<SignIn>,
    },
}

impl Flow {
    fn id(&self) -> u64 {
        match self {
            Self::Running { id, .. } | Self::Waiting { id, .. } => *id,
        }
    }
}

impl SignIns {
    pub fn begin(&mut self, agent: Agent) -> (u64, oneshot::Receiver<()>) {
        self.cancel(agent);
        self.next += 1;
        let (cancel, stop) = oneshot::channel();
        self.flows.insert(
            agent,
            Flow::Running {
                id: self.next,
                cancel,
            },
        );

        (self.next, stop)
    }

    pub fn resume(&mut self, agent: Agent) -> Option<(u64, SignIn, oneshot::Receiver<()>)> {
        match self.flows.remove(&agent) {
            Some(Flow::Waiting { id, signin }) => {
                let (cancel, stop) = oneshot::channel();
                self.flows.insert(agent, Flow::Running { id, cancel });
                Some((id, *signin, stop))
            }
            Some(flow) => {
                self.flows.insert(agent, flow);
                None
            }
            None => None,
        }
    }

    pub fn wait(&mut self, agent: Agent, id: u64, signin: SignIn) -> bool {
        if !self.owns(agent, id) {
            return false;
        }

        self.flows.insert(
            agent,
            Flow::Waiting {
                id,
                signin: Box::new(signin),
            },
        );
        true
    }

    pub fn end(&mut self, agent: Agent, id: u64) {
        if self.owns(agent, id) {
            self.flows.remove(&agent);
        }
    }

    pub fn cancel(&mut self, agent: Agent) {
        if let Some(Flow::Running { cancel, .. }) = self.flows.remove(&agent) {
            let _ = cancel.send(());
        }
    }

    fn owns(&self, agent: Agent, id: u64) -> bool {
        self.flows.get(&agent).is_some_and(|flow| flow.id() == id)
    }
}
