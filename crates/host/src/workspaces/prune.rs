use std::path::Path;

use protocol::terminals::Status;
use protocol::workspaces::{Checkout, Tile};
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
    let orphans = orphans(&workspace.tiles, checkouts);
    if orphans == Orphans::default() {
        return workspace;
    }

    let mut changed = false;
    for tile in &orphans.tiles {
        let closed = store::close(pool, tile).await;
        if let Err(error) = &closed {
            tracing::warn!(?error, "could not close a tile in a removed worktree");
        }
        changed |= closed.is_ok();
    }
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
    let closed = crate::terminals::close(session, &orphans.terminals).await;
    if let Err(error) = &closed {
        tracing::warn!(
            ?error,
            "could not close the terminals in a removed worktree"
        );
    }
    changed |= closed.is_ok() && !orphans.terminals.is_empty();

    settle(&mut workspace.tiles, &orphans);
    if changed {
        watch::announce(&session.shared);
    }

    workspace
}

fn settle(tiles: &mut Vec<Tile>, orphans: &Orphans) {
    tiles.retain(|tile| {
        let id = id(tile);
        !orphans.tiles.iter().any(|closed| closed == id)
            && !orphans.terminals.iter().any(|closed| closed == id)
    });

    for tile in tiles {
        if let Tile::Terminal(terminal) = tile
            && orphans.kept.contains(&terminal.id)
        {
            terminal.checkout = None;
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Orphans {
    tiles: Vec<String>,
    terminals: Vec<String>,
    kept: Vec<String>,
}

fn orphans(tiles: &[Tile], checkouts: &[Checkout]) -> Orphans {
    let mut orphans = Orphans::default();
    if checkouts.is_empty() {
        return orphans;
    }

    for tile in tiles {
        let checkout = match tile {
            Tile::Terminal(terminal) => terminal.checkout.as_deref(),
            Tile::Note(note) => note.checkout.as_deref(),
            Tile::Monitor(monitor) => monitor.checkout.as_deref(),
        };
        let Some(checkout) = checkout else {
            continue;
        };
        if checkouts
            .iter()
            .any(|listed| folders::same(Path::new(checkout), Path::new(&listed.path)))
        {
            continue;
        }

        let id = id(tile).to_owned();
        match tile {
            Tile::Terminal(terminal)
                if matches!(terminal.status, Status::Working | Status::Blocked) =>
            {
                orphans.kept.push(id);
            }
            Tile::Terminal(_) => orphans.terminals.push(id),
            Tile::Note(_) | Tile::Monitor(_) => orphans.tiles.push(id),
        }
    }

    orphans
}

fn id(tile: &Tile) -> &str {
    match tile {
        Tile::Terminal(terminal) => &terminal.id,
        Tile::Note(note) => &note.id,
        Tile::Monitor(monitor) => &monitor.id,
    }
}

#[cfg(test)]
mod tests {
    use protocol::terminals::Terminal;
    use protocol::workspaces::{MonitorTile, NoteTile};

    use super::*;

    fn checkout(path: &Path, main: bool) -> Checkout {
        Checkout {
            path: path.display().to_string(),
            branch: None,
            main,
        }
    }

    fn terminal(id: &str, status: Status, checkout: Option<&Path>) -> Tile {
        Tile::Terminal(Terminal {
            id: id.to_owned(),
            name: "Terminal".to_owned(),
            agent: None,
            model: None,
            status,
            folder: "/repos/app".to_owned(),
            branch: None,
            changes: None,
            checkout: checkout.map(|path| path.display().to_string()),
        })
    }

    fn note(id: &str, checkout: Option<&Path>) -> Tile {
        Tile::Note(NoteTile {
            id: id.to_owned(),
            checkout: checkout.map(|path| path.display().to_string()),
            note: None,
        })
    }

    fn monitor(id: &str, checkout: Option<&Path>) -> Tile {
        Tile::Monitor(MonitorTile {
            id: id.to_owned(),
            checkout: checkout.map(|path| path.display().to_string()),
        })
    }

    #[test]
    fn a_removed_worktree_closes_its_tiles_but_keeps_busy_agents() {
        let root = std::env::temp_dir();
        let repo = root.join("app");
        let login = root.join("worktrees").join("login");
        let gone = root.join("worktrees").join("search");
        let checkouts = [checkout(&repo, true), checkout(&login, false)];
        let tiles = [
            terminal("main", Status::Idle, None),
            terminal("login", Status::Idle, Some(&login)),
            note("login-note", Some(&login)),
            terminal("idle", Status::Idle, Some(&gone)),
            terminal("done", Status::Done, Some(&gone)),
            terminal("working", Status::Working, Some(&gone)),
            terminal("blocked", Status::Blocked, Some(&gone)),
            note("note", Some(&gone)),
            monitor("monitor", Some(&gone)),
        ];

        assert_eq!(
            orphans(&tiles, &checkouts),
            Orphans {
                tiles: vec!["note".to_owned(), "monitor".to_owned()],
                terminals: vec!["idle".to_owned(), "done".to_owned()],
                kept: vec!["working".to_owned(), "blocked".to_owned()],
            }
        );
    }

    #[test]
    fn a_busy_agent_leaves_its_removed_worktree() {
        let gone = std::env::temp_dir().join("worktrees").join("search");
        let mut tiles = vec![
            terminal("working", Status::Working, Some(&gone)),
            terminal("idle", Status::Idle, Some(&gone)),
            note("note", Some(&gone)),
        ];
        let orphans = Orphans {
            tiles: vec!["note".to_owned()],
            terminals: vec!["idle".to_owned()],
            kept: vec!["working".to_owned()],
        };

        settle(&mut tiles, &orphans);

        let [Tile::Terminal(kept)] = tiles.as_slice() else {
            panic!("expected only the busy terminal, got {tiles:?}");
        };
        assert_eq!(kept.id, "working");
        assert_eq!(kept.checkout, None);
    }

    #[test]
    fn an_unreadable_listing_closes_nothing() {
        let login = std::env::temp_dir().join("worktrees").join("login");
        let tiles = [
            terminal("idle", Status::Idle, Some(&login)),
            note("note", Some(&login)),
        ];

        assert_eq!(orphans(&tiles, &[]), Orphans::default());
    }
}
