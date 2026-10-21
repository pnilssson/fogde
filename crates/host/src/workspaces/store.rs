use std::path::Path;

use protocol::terminals::Terminal;
use sqlx::{SqliteConnection, SqlitePool};

use crate::folders;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the workspaces: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("a workspace needs a name")]
    Name,
    #[error("that workspace no longer exists")]
    Missing,
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub terminals: Vec<Terminal>,
}

pub struct Listing {
    pub workspaces: Vec<Workspace>,
    pub loose: Vec<String>,
}

pub async fn list(pool: &SqlitePool, terminals: &[Terminal]) -> Result<Listing, Error> {
    let mut tx = pool.begin().await?;

    let known = sqlx::query_scalar!("SELECT terminal FROM members")
        .fetch_all(&mut *tx)
        .await?;
    for id in known
        .iter()
        .filter(|id| !terminals.iter().any(|terminal| &terminal.id == *id))
    {
        sqlx::query!("DELETE FROM members WHERE terminal = ?", id)
            .execute(&mut *tx)
            .await?;
    }

    let mut workspaces = stored(&mut tx).await?;
    let members = sqlx::query!("SELECT workspace, terminal FROM members ORDER BY id")
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;

    let loose = terminals
        .iter()
        .filter(|terminal| !members.iter().any(|member| member.terminal == terminal.id))
        .map(|terminal| terminal.id.clone())
        .collect();
    for member in members {
        let terminal = terminals
            .iter()
            .find(|terminal| terminal.id == member.terminal);
        let workspace = workspaces
            .iter_mut()
            .find(|(id, _)| *id == member.workspace);
        if let (Some(terminal), Some((_, workspace))) = (terminal, workspace) {
            workspace.terminals.push(terminal.clone());
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

pub async fn create(pool: &SqlitePool, folder: &str) -> Result<Workspace, Error> {
    let mut tx = pool.begin().await?;
    let existing = stored(&mut tx)
        .await?
        .into_iter()
        .find(|(_, workspace)| folders::same(Path::new(&workspace.folder), Path::new(folder)));
    if let Some((_, workspace)) = existing {
        return Ok(workspace);
    }

    let name = folder_name(folder);
    let id = insert(&mut tx, &name, folder).await?;
    tx.commit().await?;

    Ok(workspace(id, name, folder.to_owned()))
}

pub async fn rename(pool: &SqlitePool, workspace: &str, name: &str) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Name);
    }

    let renamed = sqlx::query!("UPDATE workspaces SET name = ? WHERE id = ?", name, id)
        .execute(pool)
        .await?;
    if renamed.rows_affected() == 0 {
        return Err(Error::Missing);
    }

    Ok(())
}

pub async fn members(pool: &SqlitePool, workspace: &str) -> Result<Vec<String>, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;

    Ok(
        sqlx::query_scalar!("SELECT terminal FROM members WHERE workspace = ?", id)
            .fetch_all(pool)
            .await?,
    )
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

pub async fn place(pool: &SqlitePool, terminal: &str, workspace: &str) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let placed = sqlx::query!(
        "INSERT INTO members (terminal, workspace) VALUES (?, ?)
         ON CONFLICT (terminal) DO UPDATE SET workspace = excluded.workspace",
        terminal,
        id
    )
    .execute(pool)
    .await;

    match placed {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(error)) if error.is_foreign_key_violation() => {
            Err(Error::Missing)
        }
        Err(error) => Err(error.into()),
    }
}

async fn stored(db: &mut SqliteConnection) -> Result<Vec<(i64, Workspace)>, Error> {
    Ok(
        sqlx::query!("SELECT id, name, folder FROM workspaces ORDER BY id")
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| (row.id, workspace(row.id, row.name, row.folder)))
            .collect(),
    )
}

fn workspace(id: i64, name: String, folder: String) -> Workspace {
    Workspace {
        id: id.to_string(),
        name,
        folder,
        terminals: Vec::new(),
    }
}

async fn insert(db: &mut SqliteConnection, name: &str, folder: &str) -> Result<i64, Error> {
    let inserted = sqlx::query!(
        "INSERT INTO workspaces (name, folder) VALUES (?, ?)",
        name,
        folder
    )
    .execute(db)
    .await?;

    Ok(inserted.last_insert_rowid())
}

fn folder_name(folder: &str) -> String {
    Path::new(folder).file_name().map_or_else(
        || folder.to_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests;
