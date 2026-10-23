use std::path::Path;

use herdr_client::{Client, Pane};
use protocol::agents::Agent;
use protocol::rpc::Message;
use protocol::terminals::{FailedEvent, TerminalFailed};
use tokio::sync::mpsc;

use crate::rpc::notify;

pub fn start(
    client: Client,
    pane: &Pane,
    agent: Agent,
    args: Vec<String>,
    outgoing: mpsc::UnboundedSender<Message>,
) {
    let kind = kind(agent);
    let name = format!("{kind}-{}", pane.workspace.to_lowercase());
    let id = pane.workspace.clone();
    let pane = pane.id.clone();

    tokio::spawn(async move {
        let Err(error) = client.start_agent(&name, kind, &pane, &args).await else {
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
    [Agent::Claude, Agent::Codex, Agent::Copilot]
        .into_iter()
        .find(|&agent| self::label(Some(agent)) == label)
}

pub fn label(agent: Option<Agent>) -> &'static str {
    match agent {
        None => "Terminal",
        Some(Agent::Claude) => "Claude",
        Some(Agent::Codex) => "Codex",
        Some(Agent::Copilot) => "GitHub Copilot",
    }
}

pub fn agent(kind: &str) -> Option<Agent> {
    [Agent::Claude, Agent::Codex, Agent::Copilot]
        .into_iter()
        .find(|&agent| self::kind(agent) == kind)
}

fn kind(agent: Agent) -> &'static str {
    match agent {
        Agent::Claude => "claude",
        Agent::Codex => "codex",
        Agent::Copilot => "copilot",
    }
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
