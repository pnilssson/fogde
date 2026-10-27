use herdr_client::{Pane, Workspace};
use protocol::agents::Agent;
use protocol::terminals::{Status, TerminalState};

use super::{Git, Model};
use crate::terminals::{launch, title};

pub(super) fn describe(
    workspace: &Workspace,
    pane: Option<&Pane>,
    previous: Option<&TerminalState>,
    viewed: bool,
    detected: bool,
    git: &Git,
    model: Option<&Model>,
) -> TerminalState {
    let reported = pane
        .and_then(|pane| pane.agent.as_deref())
        .and_then(launch::agent);
    let agent = if detected {
        reported
    } else {
        reported.or_else(|| launch::launched(&workspace.label))
    };
    let name = name(agent, pane, previous);
    let herdr = pane.map_or(herdr_client::Status::Unknown, |pane| pane.status);
    let folder = pane
        .and_then(|pane| pane.cwd.as_ref())
        .map(|cwd| cwd.display().to_string())
        .unwrap_or_default();

    TerminalState {
        id: workspace.id.clone(),
        name,
        agent,
        model: model
            .filter(|model| Some(model.agent) == agent)
            .and_then(|model| model.name.clone()),
        status: status(herdr, previous.map(|state| state.status), viewed),
        branch: git.branches.get(&folder).cloned().flatten(),
        changes: git.changes.get(&folder).cloned(),
        folder,
    }
}

fn name(agent: Option<Agent>, pane: Option<&Pane>, previous: Option<&TerminalState>) -> String {
    let Some(agent) = agent else {
        return launch::label(None).to_owned();
    };

    pane.and_then(|pane| pane.title.as_deref().zip(pane.stripped.as_deref()))
        .and_then(|(title, stripped)| title::task(agent, title, stripped))
        .or_else(|| {
            previous
                .filter(|state| state.agent == Some(agent))
                .map(|state| state.name.clone())
        })
        .unwrap_or_else(|| launch::label(Some(agent)).to_owned())
}

pub(super) fn status(
    herdr: herdr_client::Status,
    previous: Option<Status>,
    viewed: bool,
) -> Status {
    use herdr_client::Status as Herdr;

    match herdr {
        Herdr::Working => Status::Working,
        Herdr::Blocked => Status::Blocked,
        _ if viewed => Status::Idle,
        Herdr::Done => Status::Done,
        Herdr::Idle if matches!(previous, Some(Status::Working | Status::Done)) => Status::Done,
        Herdr::Idle | Herdr::Unknown => Status::Idle,
    }
}
