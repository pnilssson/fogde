use protocol::terminals::Terminal;
use protocol::workspaces::Tile;
use sqlx::SqlitePool;

use super::{Workspace, list, place};

pub fn terminal(id: &str, folder: &str) -> Terminal {
    Terminal {
        id: id.to_owned(),
        name: format!("Terminal {id}"),
        agent: None,
        status: protocol::terminals::Status::Idle,
        folder: folder.to_owned(),
        branch: None,
        changes: None,
        checkout: None,
    }
}

pub fn id(tile: &Tile) -> &str {
    match tile {
        Tile::Terminal(terminal) => &terminal.id,
        Tile::Note(note) => &note.id,
    }
}

pub fn checkout(tile: &Tile) -> Option<&str> {
    match tile {
        Tile::Terminal(terminal) => terminal.checkout.as_deref(),
        Tile::Note(note) => note.checkout.as_deref(),
    }
}

pub fn ids(workspace: &Workspace) -> Vec<&str> {
    workspace.tiles.iter().map(id).collect()
}

pub fn terminals(workspace: &Workspace) -> Vec<&Terminal> {
    workspace
        .tiles
        .iter()
        .filter_map(|tile| match tile {
            Tile::Terminal(terminal) => Some(terminal),
            Tile::Note(_) => None,
        })
        .collect()
}

pub fn notes(workspace: &Workspace) -> Vec<Option<&str>> {
    workspace
        .tiles
        .iter()
        .filter_map(|tile| match tile {
            Tile::Note(note) => Some(note.note.as_deref()),
            Tile::Terminal(_) => None,
        })
        .collect()
}

pub async fn placed(pool: &SqlitePool, workspace: &Workspace, terminals: &[&str]) {
    for id in terminals {
        place(pool, id, &workspace.id, None).await.unwrap();
    }
}

pub async fn order(pool: &SqlitePool, checkout: Option<&str>, terminals: &[&str]) -> Vec<String> {
    let running: Vec<_> = terminals
        .iter()
        .map(|id| terminal(id, "/repos/app"))
        .collect();

    list(pool, &running).await.unwrap().workspaces[0]
        .tiles
        .iter()
        .filter(|tile| self::checkout(tile) == checkout)
        .map(|tile| id(tile).to_owned())
        .collect()
}
