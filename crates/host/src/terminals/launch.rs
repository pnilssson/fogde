use std::path::Path;

use herdr_client::{Herdr, Pane};
use protocol::agents::Agent;
use protocol::rpc::Failure;

use crate::folders;
use crate::rpc::failure;

pub async fn start(herdr: &Herdr, pane: &Pane, agent: Agent) -> Result<(), Failure> {
    let kind = match agent {
        Agent::Claude => "claude",
        Agent::Codex => "codex",
        Agent::Copilot => "copilot",
    };
    let name = format!("{kind}-{}", pane.workspace.to_lowercase());

    match herdr.client().start_agent(&name, kind, &pane.id, &[]).await {
        Ok(_) => Ok(()),
        Err(herdr_client::Error::Herdr { code, .. }) if code == "agent_not_ready" => Ok(()),
        Err(error) => Err(failure(error)),
    }
}

pub fn label(folder: &Path, home: &Path, agent: Option<Agent>) -> String {
    let place = if folders::same(folder, home) {
        "~".to_owned()
    } else {
        folder.file_name().map_or_else(
            || folder.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    };
    let what = match agent {
        None => "Terminal",
        Some(Agent::Claude) => "Claude",
        Some(Agent::Codex) => "Codex",
        Some(Agent::Copilot) => "Copilot",
    };

    format!("{place} · {what}")
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
