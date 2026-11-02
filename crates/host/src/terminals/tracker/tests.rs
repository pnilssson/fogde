use herdr_client::{Foreground, Status as Reported};
use protocol::agents::Agent;
use protocol::terminals::Status;

use super::describe::{active, status};
use super::*;

#[cfg(windows)]
const SHELL: &str = r"C:\Program Files\PowerShell\7\pwsh.exe";
#[cfg(unix)]
const SHELL: &str = "/bin/zsh";

fn terminal(label: &str) -> Terminal {
    Terminal {
        address: "t1".into(),
        labels: BTreeMap::from([
            ("fogde".to_owned(), label.to_owned()),
            ("herdr.seen".to_owned(), "0".to_owned()),
        ]),
        cwd: "/repos/app".into(),
        pid: Some(4242),
        ..Terminal::default()
    }
}

fn running(mut terminal: Terminal, process: &str, title: &str) -> Terminal {
    terminal.foreground = Some(Foreground {
        pid: 4343,
        name: process.into(),
        cwd: "/repos/app".into(),
    });
    terminal.title = title.into();
    terminal
}

fn reporting(mut terminal: Terminal, state: &str, turn: u64) -> Terminal {
    terminal.status = Some(Reported {
        state: state.into(),
        app: Some("claude-code".into()),
    });
    terminal.turn = turn;
    terminal
}

fn seen(mut terminal: Terminal, turn: u64) -> Terminal {
    terminal
        .labels
        .insert("herdr.seen".into(), turn.to_string());
    terminal
}

fn update(tracker: &Tracker, terminal: Terminal) -> Vec<(String, PathBuf)> {
    tracker.update(&BTreeMap::from([("t1".to_owned(), terminal)]))
}

fn current(tracker: &Tracker) -> TerminalState {
    lock(&tracker.states)["t1"].clone()
}

#[test]
fn views_count_once_per_session() {
    let tracker = Tracker::default();

    tracker.view(1, Some("t1".into()));
    tracker.view(2, Some("t1".into()));
    tracker.view(1, None);

    assert_eq!(tracker.viewed(), HashSet::from(["t1".to_owned()]));
    tracker.view(2, Some("t2".into()));
    assert_eq!(tracker.viewed(), HashSet::from(["t2".to_owned()]));
}

#[tokio::test]
async fn a_session_that_falls_behind_gets_every_current_state() {
    let tracker = Tracker::default();
    let mut changes = tracker.changes();
    let states: Vec<_> = (0..300)
        .map(|index| TerminalState {
            id: format!("t{index}"),
            name: "Claude".into(),
            agent: None,
            model: None,
            status: Status::Working,
            active: None,
            folder: "/repos/app".into(),
            branch: Some("main".into()),
            changes: None,
        })
        .collect();
    for state in &states {
        let _ = tracker.changes.send(state.clone());
        lock(&tracker.states).insert(state.id.clone(), state.clone());
    }

    let mut received = tracker.next(&mut changes).await.unwrap_or_default();
    received.sort_by(|a, b| a.id.cmp(&b.id));
    let mut expected = states;
    expected.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(received, expected);
}

#[test]
fn a_launched_agent_shows_until_it_exits() {
    let tracker = Tracker::default();
    let claude = terminal("Claude");

    update(&tracker, running(claude.clone(), "pwsh.exe", SHELL));
    let starting = current(&tracker);
    update(
        &tracker,
        running(claude.clone(), "claude.EXE", "Claude Code"),
    );
    let started = current(&tracker);
    update(&tracker, running(claude, "pwsh.exe", SHELL));
    let exited = current(&tracker);

    assert_eq!(
        [starting.agent, started.agent, exited.agent],
        [Some(Agent::Claude), Some(Agent::Claude), None]
    );
    assert_eq!([&*started.name, &*exited.name], ["Claude", "Terminal"]);
}

#[test]
fn an_agent_is_found_by_what_it_reports_or_its_process() {
    let tracker = Tracker::default();
    let mut codex = terminal("PowerShell");
    codex.status = Some(Reported {
        state: "idle".into(),
        app: Some("codex".into()),
    });

    update(&tracker, codex);
    let reported = current(&tracker).agent;
    update(
        &tracker,
        running(terminal("PowerShell"), "copilot", "GitHub Copilot"),
    );
    let found = current(&tracker).agent;

    assert_eq!(
        [reported, found],
        [Some(Agent::Codex), Some(Agent::Copilot)]
    );
}

#[test]
fn an_agent_is_named_after_its_task_until_it_exits() {
    let tracker = Tracker::default();
    let codex = terminal("Codex");
    let names = [
        running(codex.clone(), "codex.exe", "app"),
        running(codex.clone(), "codex.exe", "Fix the login | app"),
        running(codex.clone(), "codex.exe", "app"),
        running(codex, "pwsh.exe", SHELL),
    ]
    .map(|terminal| {
        update(&tracker, terminal);
        current(&tracker).name
    });

    assert_eq!(
        names,
        ["Codex", "Fix the login", "Fix the login", "Terminal"]
    );
}

#[test]
fn a_shell_keeps_its_name_around_an_agent_run_in_it() {
    let tracker = Tracker::default();
    let shell = terminal("PowerShell");
    let names = [
        running(shell.clone(), "pwsh.exe", SHELL),
        running(shell.clone(), "claude.exe", "Claude Code"),
        running(shell, "pwsh.exe", SHELL),
    ]
    .map(|terminal| {
        update(&tracker, terminal);
        current(&tracker).name
    });

    assert_eq!(names, ["PowerShell", "Claude", "PowerShell"]);
}

#[test]
fn a_new_agent_drops_the_last_task() {
    let tracker = Tracker::default();
    let shell = terminal("Terminal");

    update(
        &tracker,
        running(shell.clone(), "claude", "✳ Rust crates in folder"),
    );
    update(&tracker, running(shell, "copilot", "GitHub Copilot"));

    assert_eq!(current(&tracker).name, "GitHub Copilot");
}

#[test]
fn a_shell_title_before_claude_starts_is_not_its_task() {
    let tracker = Tracker::default();
    let claude = terminal("Claude");
    let names = [
        running(claude.clone(), "pwsh.exe", SHELL),
        running(claude.clone(), "claude", "Claude Code"),
        running(claude, "claude", "✳ Rust crates in folder"),
    ]
    .map(|terminal| {
        update(&tracker, terminal);
        current(&tracker).name
    });

    assert_eq!(names, ["Claude", "Claude", "Rust crates in folder"]);
}

#[test]
fn a_terminal_lives_in_the_folder_its_program_is_in() {
    let tracker = Tracker::default();
    let mut moved = running(terminal("PowerShell"), "pwsh.exe", SHELL);
    if let Some(process) = &mut moved.foreground {
        process.cwd = "/repos/app/src".into();
    }

    update(&tracker, moved);

    assert_eq!(current(&tracker).folder, "/repos/app/src");
}

#[tokio::test]
async fn an_agent_whose_process_is_unknown_waits_for_its_next_status() {
    let tracker = Tracker::default();
    let mut claude = reporting(terminal("Claude"), "idle", 0);
    claude.foreground = None;

    tracker
        .read_models(&BTreeMap::from([("t1".to_owned(), claude)]))
        .await;

    assert!(
        lock(&tracker.models)
            .get("t1")
            .is_some_and(|model| model.name.is_none())
    );
}

#[test]
fn a_turn_starts_with_a_prompt_and_not_after_a_permission_answer() {
    let tracker = Tracker::default();
    let claude = terminal("Claude");
    let started = [
        ("idle", 0),
        ("working", 0),
        ("blocked", 0),
        ("working", 0),
        ("idle", 1),
        ("working", 1),
    ]
    .map(|(state, turn)| update(&tracker, reporting(claude.clone(), state, turn)).len());

    assert_eq!(started, [0, 1, 0, 0, 0, 1]);
}

mod status;
