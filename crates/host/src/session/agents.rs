use agents::State;
use protocol::agents::{Agent, AgentState, AgentStatus, InstallAgent, ListAgents, SignInAgent};
use protocol::rpc::{Failure, SERVER_ERROR};
use serde_json::Value;

use super::{Session, failure, parse, reply};

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

    pub(super) async fn sign_in_agent(&self, params: Value) -> Result<Value, Failure> {
        let agent = from_protocol(parse::<SignInAgent>(params)?);
        if agent.uses_github() && !self.github().await? {
            return Err(Failure::new(
                SERVER_ERROR,
                format!("{agent} signs in through GitHub, so set up GitHub first"),
            ));
        }

        reply::<SignInAgent, _>(agent.sign_in().await)
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
