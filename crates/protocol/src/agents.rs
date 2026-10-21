use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
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

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentPrompt {
    Done,
    Browser,
    Code { code: String, url: String },
    Paste { url: String },
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct FinishAgentSignInParams {
    pub agent: Agent,
    pub code: Option<String>,
}

pub struct ListAgents;

impl Method for ListAgents {
    const NAME: &'static str = "agents.list";
    type Params = ();
    type Result = Vec<AgentStatus>;
}

pub struct StartAgentSignIn;

impl Method for StartAgentSignIn {
    const NAME: &'static str = "agents.startSignIn";
    type Params = Agent;
    type Result = AgentPrompt;
}

pub struct FinishAgentSignIn;

impl Method for FinishAgentSignIn {
    const NAME: &'static str = "agents.finishSignIn";
    type Params = FinishAgentSignInParams;
    type Result = ();
}

pub struct CancelAgentSignIn;

impl Method for CancelAgentSignIn {
    const NAME: &'static str = "agents.cancelSignIn";
    type Params = Agent;
    type Result = ();
}

pub struct InstallAgent;

impl Method for InstallAgent {
    const NAME: &'static str = "agents.install";
    type Params = Agent;
    type Result = ();
}
