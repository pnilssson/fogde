use std::collections::HashMap;

use agents::{Prompt, SignIn, State};
use protocol::agents::{
    Agent, AgentPrompt, AgentState, AgentStatus, CancelAgentSignIn, FinishAgentSignIn,
    InstallAgent, ListAgents, StartAgentSignIn,
};
use protocol::rpc::{Failure, SERVER_ERROR};
use serde_json::Value;
use tokio::sync::oneshot;

use super::{Session, cancelled, failure, parse, reply};

#[derive(Default)]
pub(super) struct SignIns {
    next: u64,
    flows: HashMap<agents::Agent, Flow>,
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
    fn begin(&mut self, agent: agents::Agent) -> (u64, oneshot::Receiver<()>) {
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

    fn resume(&mut self, agent: agents::Agent) -> Option<(u64, SignIn, oneshot::Receiver<()>)> {
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

    fn wait(&mut self, agent: agents::Agent, id: u64, signin: SignIn) -> bool {
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

    fn end(&mut self, agent: agents::Agent, id: u64) {
        if self.owns(agent, id) {
            self.flows.remove(&agent);
        }
    }

    fn cancel(&mut self, agent: agents::Agent) {
        if let Some(Flow::Running { cancel, .. }) = self.flows.remove(&agent) {
            let _ = cancel.send(());
        }
    }

    fn owns(&self, agent: agents::Agent, id: u64) -> bool {
        self.flows.get(&agent).is_some_and(|flow| flow.id() == id)
    }
}

impl Session {
    pub(super) async fn agents(&self, params: Value) -> Result<Value, Failure> {
        parse::<ListAgents>(params)?;
        let states = agents::states().await.map_err(|error| failure(&error))?;
        let github = !states
            .iter()
            .any(|(agent, state)| agent.uses_github() && *state == State::SignedIn)
            || self.github().await?;

        reply::<ListAgents, agents::Error>(Ok(states
            .into_iter()
            .map(|(agent, state)| AgentStatus {
                agent: to_protocol(agent),
                state: match state {
                    State::Missing => AgentState::Missing,
                    State::SignedIn if github || !agent.uses_github() => AgentState::SignedIn,
                    State::SignedIn | State::SignedOut => AgentState::SignedOut,
                },
            })
            .collect()))
    }

    pub(super) async fn start_agent_sign_in(&self, params: Value) -> Result<Value, Failure> {
        let agent = from_protocol(parse::<StartAgentSignIn>(params)?);
        if agent.uses_github() && !self.github().await? {
            return Err(Failure::new(
                SERVER_ERROR,
                format!("{agent} signs in through GitHub, so set up GitHub first"),
            ));
        }
        let (id, stop) = self.agent_sign_ins.lock().await.begin(agent);

        let started = tokio::select! {
            started = SignIn::start(agent, !self.local) => started,
            _ = stop => return Err(cancelled()),
            () = self.closed() => return Err(cancelled()),
        };

        let mut sign_ins = self.agent_sign_ins.lock().await;
        let signin = match started {
            Ok(Some(signin)) => signin,
            Ok(None) => {
                sign_ins.end(agent, id);
                return reply::<StartAgentSignIn, agents::Error>(Ok(AgentPrompt::Done));
            }
            Err(error) => {
                sign_ins.end(agent, id);
                return Err(failure(&error));
            }
        };
        let prompt = match &signin.prompt {
            Prompt::Browser => AgentPrompt::Browser,
            Prompt::Code { code, url } => AgentPrompt::Code {
                code: code.clone(),
                url: url.clone(),
            },
            Prompt::Paste { url } => AgentPrompt::Paste { url: url.clone() },
        };
        let waiting = sign_ins.wait(agent, id, signin);
        drop(sign_ins);

        if !waiting {
            return Err(cancelled());
        }

        reply::<StartAgentSignIn, agents::Error>(Ok(prompt))
    }

    pub(super) async fn finish_agent_sign_in(&self, params: Value) -> Result<Value, Failure> {
        let params = parse::<FinishAgentSignIn>(params)?;
        let agent = from_protocol(params.agent);
        let Some((id, signin, stop)) = self.agent_sign_ins.lock().await.resume(agent) else {
            return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
        };

        let result = tokio::select! {
            result = signin.finish(params.code.as_deref()) => result,
            _ = stop => return Err(cancelled()),
            () = self.closed() => return Err(cancelled()),
        };
        self.agent_sign_ins.lock().await.end(agent, id);

        reply::<FinishAgentSignIn, _>(result)
    }

    pub(super) async fn cancel_agent_sign_in(&self, params: Value) -> Result<Value, Failure> {
        let agent = from_protocol(parse::<CancelAgentSignIn>(params)?);
        self.agent_sign_ins.lock().await.cancel(agent);

        reply::<CancelAgentSignIn, agents::Error>(Ok(()))
    }

    pub(super) async fn install_agent(&self, params: Value) -> Result<Value, Failure> {
        let agent = from_protocol(parse::<InstallAgent>(params)?);

        reply::<InstallAgent, _>(agent.install().await)
    }

    async fn github(&self) -> Result<bool, Failure> {
        let accounts = self
            .gh()
            .await?
            .accounts()
            .await
            .map_err(|error| failure(&error))?;

        Ok(!accounts.is_empty())
    }
}

fn to_protocol(agent: agents::Agent) -> Agent {
    match agent {
        agents::Agent::Claude => Agent::Claude,
        agents::Agent::Codex => Agent::Codex,
        agents::Agent::Copilot => Agent::Copilot,
    }
}

fn from_protocol(agent: Agent) -> agents::Agent {
    match agent {
        Agent::Claude => agents::Agent::Claude,
        Agent::Codex => agents::Agent::Codex,
        Agent::Copilot => agents::Agent::Copilot,
    }
}
