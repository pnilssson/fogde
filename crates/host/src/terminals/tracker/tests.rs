use herdr_client::Status as Herdr;
use protocol::agents::Agent;

use super::*;

#[test]
fn an_agent_that_finishes_unseen_is_done_until_viewed() {
    let finished = status(Herdr::Idle, Some(Status::Working), false);
    let still = status(Herdr::Idle, Some(finished), false);
    let seen = status(Herdr::Idle, Some(still), true);
    let later = status(Herdr::Idle, Some(seen), false);

    assert_eq!(
        [finished, still, seen, later],
        [Status::Done, Status::Done, Status::Idle, Status::Idle]
    );
}

#[test]
fn herdr_done_clears_only_when_viewed() {
    assert_eq!(status(Herdr::Done, None, false), Status::Done);
    assert_eq!(status(Herdr::Done, Some(Status::Done), true), Status::Idle);
}

#[test]
fn an_agent_waiting_for_input_stays_blocked_while_viewed() {
    assert_eq!(status(Herdr::Blocked, None, true), Status::Blocked);
    assert_eq!(status(Herdr::Working, None, true), Status::Working);
}

#[test]
fn an_agent_seen_finishing_is_idle() {
    assert_eq!(
        status(Herdr::Idle, Some(Status::Working), true),
        Status::Idle
    );
    assert_eq!(
        status(Herdr::Unknown, Some(Status::Done), false),
        Status::Idle
    );
}

#[test]
fn views_count_once_per_session() {
    let tracker = Tracker::default();

    tracker.view(1, Some("w1".into()));
    tracker.view(2, Some("w1".into()));
    tracker.view(1, None);

    assert_eq!(tracker.viewed(), HashSet::from(["w1".to_owned()]));
    tracker.view(2, Some("w2".into()));
    assert_eq!(tracker.viewed(), HashSet::from(["w2".to_owned()]));
}

#[tokio::test]
async fn a_session_that_falls_behind_gets_every_current_state() {
    let tracker = Tracker::default();
    let mut changes = tracker.changes();
    let states: Vec<_> = (0..300)
        .map(|index| TerminalState {
            id: format!("w{index}"),
            name: "Claude".into(),
            agent: None,
            status: Status::Working,
            folder: "/repos/app".into(),
            branch: Some("main".into()),
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

fn workspace(label: &str) -> Workspace {
    Workspace {
        id: "w1".into(),
        label: label.into(),
    }
}

fn pane(agent: Option<&str>) -> Pane {
    Pane {
        id: "w1:p1".into(),
        workspace: "w1".into(),
        agent: agent.map(Into::into),
        status: Herdr::Idle,
        session: None,
        cwd: None,
    }
}

fn current(tracker: &Tracker) -> TerminalState {
    lock(&tracker.states)["w1"].clone()
}

#[test]
fn a_launched_agent_shows_until_herdr_reports_it_gone() {
    let tracker = Tracker::default();
    let claude = workspace("Claude");

    tracker.update(std::slice::from_ref(&claude), &[pane(None)]);
    let starting = current(&tracker);
    tracker.update(std::slice::from_ref(&claude), &[pane(Some("claude"))]);
    let running = current(&tracker);
    tracker.update(std::slice::from_ref(&claude), &[pane(None)]);
    let exited = current(&tracker);

    assert_eq!(
        [starting.agent, running.agent, exited.agent],
        [Some(Agent::Claude), Some(Agent::Claude), None]
    );
    assert_eq!([&*running.name, &*exited.name], ["Claude", "Terminal"]);
}

#[test]
fn a_custom_name_stays_when_the_agent_changes() {
    let tracker = Tracker::default();

    tracker.update(&[workspace("Fix the login")], &[pane(Some("codex"))]);

    assert_eq!(current(&tracker).name, "Fix the login");
    assert_eq!(current(&tracker).agent, Some(Agent::Codex));
}
