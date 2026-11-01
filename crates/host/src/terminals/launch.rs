use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use protocol::agents::Agent;

#[derive(Clone, Default)]
pub struct Launches(Arc<Mutex<HashMap<String, Agent>>>);

impl Launches {
    pub fn hold(&self, id: String, agent: Agent) {
        self.lock().insert(id, agent);
    }

    pub fn take(&self, id: &str) -> Option<Agent> {
        self.lock().remove(id)
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Agent>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn command(agent: Agent) -> String {
    let spec = agent.spec();
    let args = match agent {
        Agent::Codex => " --no-daemon",
        Agent::Claude | Agent::Copilot => "",
    };

    format!("{}{args}\r", spec.command)
}

pub fn launched(label: &str) -> Option<Agent> {
    Agent::ALL
        .into_iter()
        .find(|agent| agent.spec().short == label)
}

pub fn shell(label: &str) -> &str {
    if launched(label).is_some() {
        return "Terminal";
    }

    label
}

pub fn agent(command: &str) -> Option<Agent> {
    Agent::ALL
        .into_iter()
        .find(|agent| agent.spec().command == command)
}

pub fn path_with(first: &[PathBuf]) -> String {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let dirs = first.iter().cloned().chain(std::env::split_paths(&current));

    std::env::join_paths(dirs).map_or_else(
        |_| current.to_string_lossy().into_owned(),
        |joined| joined.to_string_lossy().into_owned(),
    )
}
