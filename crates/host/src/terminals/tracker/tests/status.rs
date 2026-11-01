use super::*;

#[test]
fn an_agent_that_finishes_unseen_is_done_until_seen() {
    let claude = terminal("Claude");

    assert_eq!(
        [
            status(&reporting(claude.clone(), "idle", 0)),
            status(&reporting(claude.clone(), "working", 0)),
            status(&reporting(claude.clone(), "done", 1)),
            status(&seen(reporting(claude.clone(), "done", 1), 1)),
            status(&seen(reporting(claude, "idle", 2), 1)),
        ],
        [
            Status::Idle,
            Status::Working,
            Status::Done,
            Status::Idle,
            Status::Done
        ]
    );
}

#[test]
fn an_agent_waiting_for_you_stays_blocked_once_seen() {
    let claude = seen(terminal("Claude"), 3);

    assert_eq!(
        status(&reporting(claude.clone(), "blocked", 3)),
        Status::Blocked
    );
    assert_eq!(status(&reporting(claude, "error", 3)), Status::Blocked);
}

#[test]
fn a_terminal_that_never_reports_stays_idle() {
    assert_eq!(status(&terminal("PowerShell")), Status::Idle);
}

#[test]
fn an_agent_is_active_when_it_starts_or_stops_working() {
    let idle = sample(Status::Idle, None);
    let working = sample(Status::Working, active(Status::Working, Some(&idle), 100));
    let done = sample(Status::Done, active(Status::Done, Some(&working), 160));
    let seen = active(Status::Idle, Some(&done), 900);

    assert_eq!(working.active, Some(100));
    assert_eq!(done.active, Some(160));
    assert_eq!(seen, Some(160));
}

#[test]
fn an_idle_shell_has_no_activity() {
    assert_eq!(active(Status::Idle, None, 100), None);
    assert_eq!(
        active(Status::Idle, Some(&sample(Status::Idle, None)), 200),
        None
    );
}
fn sample(status: Status, active: Option<i64>) -> TerminalState {
    TerminalState {
        id: "t1".into(),
        name: "Claude".into(),
        agent: Some(Agent::Claude),
        model: None,
        status,
        active,
        folder: "/repos/app".into(),
        branch: Some("main".into()),
        changes: None,
    }
}
