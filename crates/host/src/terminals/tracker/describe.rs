use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use herdr_client::Terminal;
use protocol::agents::Agent;
use protocol::terminals::{Status, TerminalState};

use super::{Git, Model};
use crate::terminals::{OWNER, SEEN, launch, title};

pub(super) fn describe(
    terminal: &Terminal,
    previous: Option<&TerminalState>,
    detected: bool,
    git: &Git,
    model: Option<&Model>,
) -> TerminalState {
    let label = terminal.labels.get(OWNER).map_or("", String::as_str);
    let agent =
        running(terminal).or_else(|| (!detected).then(|| launch::launched(label)).flatten());
    let folder = folder(terminal);
    let status = status(terminal);

    TerminalState {
        id: terminal.address.clone(),
        name: name(terminal, label, agent, previous),
        agent,
        model: model
            .filter(|model| Some(model.agent) == agent)
            .and_then(|model| model.name.clone()),
        status,
        active: active(status, previous, now()),
        branch: git.branches.get(&folder).cloned().flatten(),
        changes: git.changes.get(&folder).cloned(),
        folder,
    }
}

pub(super) fn running(terminal: &Terminal) -> Option<Agent> {
    let reported = terminal
        .status
        .as_ref()
        .and_then(|status| status.app.as_deref())
        .and_then(|app| match app {
            "claude-code" => Some(Agent::Claude),
            app => launch::agent(app),
        });

    reported.or_else(|| {
        let process = terminal.foreground.as_ref()?;
        let stem = Path::new(&process.name).file_stem()?.to_str()?;

        launch::agent(&stem.to_lowercase())
    })
}

pub(super) fn folder(terminal: &Terminal) -> String {
    terminal
        .foreground
        .as_ref()
        .map(|process| process.cwd.as_str())
        .filter(|cwd| !cwd.is_empty())
        .unwrap_or(&terminal.cwd)
        .to_owned()
}

pub(super) fn seen(terminal: &Terminal) -> Option<u64> {
    terminal.labels.get(SEEN)?.parse().ok()
}

pub(super) fn status(terminal: &Terminal) -> Status {
    let state = terminal
        .status
        .as_ref()
        .map_or("", |status| status.state.as_str());

    match state {
        "blocked" | "error" => Status::Blocked,
        "working" => Status::Working,
        _ if seen(terminal).is_some_and(|seen| terminal.turn > seen) => Status::Done,
        _ => Status::Idle,
    }
}

fn name(
    terminal: &Terminal,
    label: &str,
    agent: Option<Agent>,
    previous: Option<&TerminalState>,
) -> String {
    let Some(agent) = agent else {
        return launch::shell(label).to_owned();
    };

    title::task(agent, &terminal.title, title::strip(&terminal.title))
        .or_else(|| {
            previous
                .filter(|state| state.agent == Some(agent))
                .map(|state| state.name.clone())
        })
        .unwrap_or_else(|| agent.spec().short.to_owned())
}

pub(super) fn active(status: Status, previous: Option<&TerminalState>, now: i64) -> Option<i64> {
    let before = previous.map(|state| state.status);
    if before != Some(status) && (status == Status::Working || before == Some(Status::Working)) {
        return Some(now);
    }

    previous.and_then(|state| state.active)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or_default()
}
