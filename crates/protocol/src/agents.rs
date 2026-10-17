use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
}

pub struct SignedInAgents;

impl Method for SignedInAgents {
    const NAME: &'static str = "agents.signedIn";
    type Params = ();
    type Result = Vec<Agent>;
}

pub struct SignInAgent;

impl Method for SignInAgent {
    const NAME: &'static str = "agents.signIn";
    type Params = Agent;
    type Result = ();
}
