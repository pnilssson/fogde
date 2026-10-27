use std::path::Path;

use protocol::terminals::Terminal;
use protocol::workspaces::{MonitorTile, NoteTile, Tile};
use sqlx::{SqliteConnection, SqlitePool};

use crate::folders;

mod tiles;

pub use tiles::{
    arrange, close, follow, members, open_monitor, open_note, place, show_note, terminals_in,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the workspaces: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("that workspace no longer exists")]
    Missing,
    #[error("that tile is closed")]
    Closed,
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub id: String,
    pub folder: String,
    pub identity: Option<String>,
    pub tiles: Vec<Tile>,
}

pub struct Listing {
    pub workspaces: Vec<Workspace>,
    pub loose: Vec<String>,
}

pub async fn list(pool: &SqlitePool, terminals: &[Terminal]) -> Result<Listing, Error> {
    let mut tx = pool.begin().await?;

    let known = sqlx::query_scalar!("SELECT tile FROM tiles WHERE kind = 'terminal'")
        .fetch_all(&mut *tx)
        .await?;
    for id in known
        .iter()
        .filter(|id| !terminals.iter().any(|terminal| &terminal.id == *id))
    {
        sqlx::query!("DELETE FROM tiles WHERE tile = ?", id)
            .execute(&mut *tx)
            .await?;
    }

    let mut workspaces = stored(&mut tx).await?;
    let rows = sqlx::query!(
        "SELECT tile, kind, note, workspace, checkout FROM tiles ORDER BY position, id"
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    let loose = terminals
        .iter()
        .filter(|terminal| {
            !rows
                .iter()
                .any(|row| row.kind == "terminal" && row.tile == terminal.id)
        })
        .map(|terminal| terminal.id.clone())
        .collect();
    for row in rows {
        let Some((_, workspace)) = workspaces.iter_mut().find(|(id, _)| *id == row.workspace)
        else {
            continue;
        };

        let tile = if row.kind == "note" {
            Some(Tile::Note(NoteTile {
                id: row.tile,
                checkout: row.checkout,
                note: row.note,
            }))
        } else if row.kind == "monitor" {
            Some(Tile::Monitor(MonitorTile {
                id: row.tile,
                checkout: row.checkout,
            }))
        } else {
            terminals
                .iter()
                .find(|terminal| terminal.id == row.tile)
                .map(|terminal| {
                    Tile::Terminal(Terminal {
                        checkout: row.checkout,
                        ..terminal.clone()
                    })
                })
        };
        if let Some(tile) = tile {
            workspace.tiles.push(tile);
        }
    }

    Ok(Listing {
        workspaces: workspaces
            .into_iter()
            .map(|(_, workspace)| workspace)
            .collect(),
        loose,
    })
}

pub async fn create(
    pool: &SqlitePool,
    folder: &str,
    identity: Option<&str>,
) -> Result<Workspace, Error> {
    let mut tx = pool.begin().await?;
    let existing = stored(&mut tx)
        .await?
        .into_iter()
        .find(|(_, workspace)| folders::same(Path::new(&workspace.folder), Path::new(folder)));
    if let Some((_, workspace)) = existing {
        return Ok(workspace);
    }

    let inserted = sqlx::query!(
        "INSERT INTO workspaces (folder, identity) VALUES (?, ?)",
        folder,
        identity
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(workspace(
        inserted.last_insert_rowid(),
        folder.to_owned(),
        identity.map(ToOwned::to_owned),
    ))
}

pub async fn relocate(
    pool: &SqlitePool,
    workspace: &str,
    folder: &str,
    identity: Option<&str>,
) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let moved = sqlx::query!(
        "UPDATE workspaces SET folder = ?, identity = ? WHERE id = ?",
        folder,
        identity,
        id
    )
    .execute(pool)
    .await?;
    if moved.rows_affected() == 0 {
        return Err(Error::Missing);
    }

    Ok(())
}

pub async fn folders(pool: &SqlitePool) -> Result<Vec<String>, Error> {
    Ok(sqlx::query_scalar!("SELECT folder FROM workspaces")
        .fetch_all(pool)
        .await?)
}

pub async fn delete(pool: &SqlitePool, workspace: &str) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let deleted = sqlx::query!("DELETE FROM workspaces WHERE id = ?", id)
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(Error::Missing);
    }

    Ok(())
}

async fn stored(db: &mut SqliteConnection) -> Result<Vec<(i64, Workspace)>, Error> {
    Ok(
        sqlx::query!("SELECT id, folder, identity FROM workspaces ORDER BY id")
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| (row.id, workspace(row.id, row.folder, row.identity)))
            .collect(),
    )
}

fn workspace(id: i64, folder: String, identity: Option<String>) -> Workspace {
    Workspace {
        id: id.to_string(),
        folder,
        identity,
        tiles: Vec::new(),
    }
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
