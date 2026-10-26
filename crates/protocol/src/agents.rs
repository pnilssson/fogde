use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Method;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Copilot,
}

#[derive(Clone, Copy, Debug)]
pub struct AgentSpec {
    pub name: &'static str,
    pub short: &'static str,
    pub account: &'static str,
    pub command: &'static str,
    pub domains: &'static [&'static str],
    pub github: bool,
}

const CLAUDE: AgentSpec = AgentSpec {
    name: "Claude Code",
    short: "Claude",
    account: "Claude",
    command: "claude",
    domains: &["claude.com", "claude.ai", "anthropic.com"],
    github: false,
};

const CODEX: AgentSpec = AgentSpec {
    name: "Codex",
    short: "Codex",
    account: "ChatGPT",
    command: "codex",
    domains: &["openai.com", "chatgpt.com"],
    github: false,
};

const COPILOT: AgentSpec = AgentSpec {
    name: "GitHub Copilot",
    short: "GitHub Copilot",
    account: "GitHub Copilot",
    command: "copilot",
    domains: &["github.com", "ghe.com"],
    github: true,
};

impl Agent {
    pub const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Copilot];

    pub const fn spec(self) -> &'static AgentSpec {
        match self {
            Self::Claude => &CLAUDE,
            Self::Codex => &CODEX,
            Self::Copilot => &COPILOT,
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.spec().name)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentState {
    Missing,
    SignedOut,
    SignedIn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentStatus {
    pub agent: Agent,
    pub state: AgentState,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentPrompt {
    Done,
    Browser,
    Code { code: String, url: String },
    Paste { url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
