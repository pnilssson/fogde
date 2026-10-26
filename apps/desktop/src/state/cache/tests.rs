use protocol::agents::Agent;
use protocol::terminals::{Status, Terminal, TerminalState};
use protocol::workspaces::{MoveTerminalParams, PlaceTerminalParams, Workspace};

use super::Cache;

fn terminal(id: &str) -> Terminal {
    Terminal {
        id: id.into(),
        name: "Claude".into(),
        agent: Some(Agent::Claude),
        status: Status::Idle,
        folder: "/repos/app".into(),
        branch: Some("main".into()),
        changes: None,
        checkout: None,
    }
}

fn workspace(id: &str, terminals: Vec<Terminal>) -> Workspace {
    Workspace {
        id: id.into(),
        name: "app".into(),
        folder: "/repos/app".into(),
        missing: false,
        checkouts: Vec::new(),
        terminals,
    }
}

fn loaded(list: Vec<Workspace>) -> Cache {
    let mut cache = Cache::default();
    let ticket = cache.begin();
    cache.finish(ticket, Ok(list));
    cache
}

fn terminals(cache: &Cache) -> Vec<String> {
    cache.list().unwrap()[0]
        .terminals
        .iter()
        .map(|terminal| terminal.id.clone())
        .collect()
}

#[test]
fn keeps_a_new_terminal_when_an_older_list_arrives_after_it() {
    let mut cache = loaded(vec![workspace("2", Vec::new())]);
    let stale = cache.begin();

    cache.add("2", terminal("w2"));
    let fresh = cache.begin();
    cache.finish(stale, Ok(vec![workspace("2", Vec::new())]));
    cache.finish(fresh, Ok(vec![workspace("2", vec![terminal("w2")])]));

    assert_eq!(terminals(&cache), ["w2"]);
}

#[test]
fn adds_a_terminal_the_list_already_shows_only_once() {
    let mut cache = loaded(vec![workspace("2", vec![terminal("w2")])]);

    cache.add("2", terminal("w2"));

    assert_eq!(terminals(&cache), ["w2"]);
}

#[test]
fn keeps_a_closed_terminal_gone_when_an_older_list_arrives_after_it() {
    let mut cache = loaded(vec![workspace("2", vec![terminal("w2")])]);
    let stale = cache.begin();

    cache.remove("w2");
    cache.finish(stale, Ok(vec![workspace("2", vec![terminal("w2")])]));

    assert_eq!(terminals(&cache), Vec::<String>::new());
}

#[test]
fn applies_a_state_change_with_its_folder_and_branch_to_the_terminal_it_names() {
    let mut cache = loaded(vec![workspace("2", vec![terminal("w2")])]);

    let applied = cache
        .apply(&TerminalState {
            id: "w2".into(),
            name: "Fix the login".into(),
            agent: Some(Agent::Claude),
            status: Status::Blocked,
            folder: "/repos/app/src".into(),
            branch: Some("fix-login".into()),
            changes: None,
        })
        .unwrap();

    assert_eq!(applied.before.status, Status::Idle);
    assert_eq!(applied.workspace.name, "app");
    assert!(applied.moved);
    let patched = &cache.list().unwrap()[0].terminals[0];
    assert_eq!(patched.name, "Fix the login");
    assert_eq!(patched.status, Status::Blocked);
    assert_eq!(patched.folder, "/repos/app/src");
    assert_eq!(patched.branch.as_deref(), Some("fix-login"));
}

#[test]
fn reports_a_terminal_it_does_not_know() {
    let mut cache = loaded(vec![workspace("2", Vec::new())]);

    let applied = cache.apply(&TerminalState {
        id: "w2".into(),
        name: "Claude".into(),
        agent: Some(Agent::Claude),
        status: Status::Working,
        folder: "/repos/app".into(),
        branch: Some("main".into()),
        changes: None,
    });

    assert!(applied.is_none());
}

#[test]
fn places_a_terminal_in_another_workspace_with_its_checkout() {
    let mut cache = loaded(vec![
        workspace("1", vec![terminal("w1")]),
        workspace("2", Vec::new()),
    ]);

    cache.place(&PlaceTerminalParams {
        terminal: "w1".into(),
        workspace: "2".into(),
        checkout: Some("/worktrees/app/login".into()),
    });

    let list = cache.list().unwrap();
    assert_eq!(list[0].terminals, Vec::new());
    assert_eq!(
        list[1].terminals[0].checkout.as_deref(),
        Some("/worktrees/app/login")
    );
}

#[test]
fn moves_a_terminal_before_another_or_last() {
    let mut cache = loaded(vec![workspace(
        "1",
        vec![terminal("a"), terminal("b"), terminal("c")],
    )]);

    cache.arrange(&MoveTerminalParams {
        terminal: "c".into(),
        before: Some("a".into()),
    });
    assert_eq!(terminals(&cache), ["c", "a", "b"]);

    cache.arrange(&MoveTerminalParams {
        terminal: "c".into(),
        before: None,
    });
    assert_eq!(terminals(&cache), ["a", "b", "c"]);
}
