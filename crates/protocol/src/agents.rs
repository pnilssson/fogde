use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Copilot,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AgentState {
    Missing,
    SignedOut,
    SignedIn,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct AgentStatus {
    pub agent: Agent,
    pub state: AgentState,
}

pub struct ListAgents;

impl Method for ListAgents {
    const NAME: &'static str = "agents.list";
    type Params = ();
    type Result = Vec<AgentStatus>;
}

pub struct SignInAgent;

impl Method for SignInAgent {
    const NAME: &'static str = "agents.signIn";
    type Params = Agent;
    type Result = ();
}

pub struct InstallAgent;

impl Method for InstallAgent {
    const NAME: &'static str = "agents.install";
    type Params = Agent;
    type Result = ();
}
