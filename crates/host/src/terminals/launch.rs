use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use herdr_client::{Client, Pane};
use protocol::agents::Agent;
use protocol::rpc::Message;
use protocol::terminals::{FailedEvent, TerminalFailed};
use tokio::sync::mpsc;

use crate::rpc::notify;

pub struct Launch {
    pub pane: Pane,
    pub agent: Agent,
}

#[derive(Clone, Default)]
pub struct Launches(Arc<Mutex<HashMap<String, Launch>>>);

impl Launches {
    pub fn hold(&self, id: String, launch: Launch) {
        self.lock().insert(id, launch);
    }

    pub fn holds(&self, id: &str) -> bool {
        self.lock().contains_key(id)
    }

    pub fn take(&self, id: &str) -> Option<Launch> {
        self.lock().remove(id)
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Launch>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn start(client: Client, launch: Launch, outgoing: mpsc::UnboundedSender<Message>) {
    let Launch { pane, agent } = launch;
    let name = format!("{}-{}", agent.spec().command, pane.workspace.to_lowercase());
    let id = pane.workspace;
    let pane = pane.id;

    tokio::spawn(async move {
        let Err(error) = client.start_agent(&name, agent.spec().command, &pane).await else {
            return;
        };

        tracing::warn!(%error, %pane, "could not start the agent");
        if let Err(error) = client.close_workspace(&id).await {
            tracing::warn!(%error, %id, "could not close the terminal");
        }
        notify::<FailedEvent>(
            &outgoing,
            TerminalFailed {
                id,
                agent,
                error: error.to_string(),
            },
        );
    });
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

pub fn path_with(program: &Path) -> String {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let dirs = program
        .parent()
        .map(Path::to_path_buf)
        .into_iter()
        .chain(std::env::split_paths(&current));

    std::env::join_paths(dirs).map_or_else(
        |_| current.to_string_lossy().into_owned(),
        |joined| joined.to_string_lossy().into_owned(),
    )
}
