use std::path::Path;

use protocol::terminals::{Status, Terminal};
use protocol::workspaces::Checkout;
use sqlx::SqlitePool;

use super::{store, watch};
use crate::folders;
use crate::session::Session;

pub async fn prune(
    session: &Session,
    pool: &SqlitePool,
    mut workspace: store::Workspace,
    checkouts: &[Checkout],
) -> store::Workspace {
    let orphans = orphans(&workspace.terminals, checkouts);
    if orphans == Orphans::default() {
        return workspace;
    }

    let mut changed = false;
    for terminal in &orphans.kept {
        let moved = store::place(pool, terminal, &workspace.id, None).await;
        if let Err(error) = &moved {
            tracing::warn!(
                ?error,
                "could not move a busy terminal out of a removed worktree"
            );
        }
        changed |= moved.is_ok();
    }
    let closed = crate::terminals::close(session, &orphans.closed).await;
    if let Err(error) = &closed {
        tracing::warn!(
            ?error,
            "could not close the terminals in a removed worktree"
        );
    }
    changed |= closed.is_ok() && !orphans.closed.is_empty();

    settle(&mut workspace.terminals, &orphans);
    if changed {
        watch::announce(&session.shared);
    }

    workspace
}

fn settle(terminals: &mut Vec<Terminal>, orphans: &Orphans) {
    terminals.retain(|terminal| !orphans.closed.contains(&terminal.id));

    for terminal in terminals {
        if orphans.kept.contains(&terminal.id) {
            terminal.checkout = None;
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Orphans {
    closed: Vec<String>,
    kept: Vec<String>,
}

fn orphans(terminals: &[Terminal], checkouts: &[Checkout]) -> Orphans {
    let mut orphans = Orphans::default();
    if checkouts.is_empty() {
        return orphans;
    }

    for terminal in terminals {
        let Some(checkout) = terminal.checkout.as_deref() else {
            continue;
        };
        if checkouts
            .iter()
            .any(|listed| folders::same(Path::new(checkout), Path::new(&listed.path)))
        {
            continue;
        }

        let id = terminal.id.clone();
        if matches!(terminal.status, Status::Working | Status::Blocked) {
            orphans.kept.push(id);
        } else {
            orphans.closed.push(id);
        }
    }

    orphans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkout(path: &Path, main: bool) -> Checkout {
        Checkout {
            path: path.display().to_string(),
            branch: None,
            main,
        }
    }

    fn terminal(id: &str, status: Status, checkout: Option<&Path>) -> Terminal {
        Terminal {
            id: id.to_owned(),
            name: "Terminal".to_owned(),
            agent: None,
            model: None,
            status,
            folder: "/repos/app".to_owned(),
            branch: None,
            changes: None,
            checkout: checkout.map(|path| path.display().to_string()),
        }
    }

    #[test]
    fn a_removed_worktree_closes_its_terminals_but_keeps_busy_agents() {
        let root = std::env::temp_dir();
        let repo = root.join("app");
        let login = root.join("worktrees").join("login");
        let gone = root.join("worktrees").join("search");
        let checkouts = [checkout(&repo, true), checkout(&login, false)];
        let terminals = [
            terminal("main", Status::Idle, None),
            terminal("login", Status::Idle, Some(&login)),
            terminal("idle", Status::Idle, Some(&gone)),
            terminal("done", Status::Done, Some(&gone)),
            terminal("working", Status::Working, Some(&gone)),
            terminal("blocked", Status::Blocked, Some(&gone)),
        ];

        assert_eq!(
            orphans(&terminals, &checkouts),
            Orphans {
                closed: vec!["idle".to_owned(), "done".to_owned()],
                kept: vec!["working".to_owned(), "blocked".to_owned()],
            }
        );
    }

    #[test]
    fn a_busy_agent_leaves_its_removed_worktree() {
        let gone = std::env::temp_dir().join("worktrees").join("search");
        let mut terminals = vec![
            terminal("working", Status::Working, Some(&gone)),
            terminal("idle", Status::Idle, Some(&gone)),
        ];
        let orphans = Orphans {
            closed: vec!["idle".to_owned()],
            kept: vec!["working".to_owned()],
        };

        settle(&mut terminals, &orphans);

        let [kept] = terminals.as_slice() else {
            panic!("expected only the busy terminal, got {terminals:?}");
        };
        assert_eq!(kept.id, "working");
        assert_eq!(kept.checkout, None);
    }

    #[test]
    fn an_unreadable_listing_closes_nothing() {
        let login = std::env::temp_dir().join("worktrees").join("login");
        let terminals = [terminal("idle", Status::Idle, Some(&login))];

        assert_eq!(orphans(&terminals, &[]), Orphans::default());
    }
}
