use std::path::Path;

use sqlx::SqlitePool;

use super::Error;
use crate::folders;

pub async fn members(pool: &SqlitePool, workspace: &str) -> Result<Vec<String>, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;

    Ok(sqlx::query_scalar!(
        "SELECT tile FROM tiles WHERE workspace = ? AND kind = 'terminal'",
        id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn place(
    pool: &SqlitePool,
    terminal: &str,
    workspace: &str,
    checkout: Option<&str>,
) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    sqlx::query!(
        "INSERT INTO tiles (tile, kind, workspace, checkout, position)
         VALUES (?, 'terminal', ?, ?, (
             SELECT COALESCE(MAX(position), 0) + 1 FROM tiles
             WHERE workspace = ? AND checkout IS ?
         ))
         ON CONFLICT (tile) DO UPDATE
         SET workspace = excluded.workspace,
             checkout = excluded.checkout,
             position = CASE
                 WHEN tiles.workspace = excluded.workspace
                     AND tiles.checkout IS excluded.checkout
                 THEN tiles.position
                 ELSE excluded.position
             END",
        terminal,
        id,
        checkout,
        id,
        checkout
    )
    .execute(pool)
    .await
    .map_err(missing)?;

    Ok(())
}

pub async fn open_note(
    pool: &SqlitePool,
    workspace: &str,
    checkout: Option<&str>,
    note: Option<&str>,
) -> Result<String, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;

    sqlx::query_scalar!(
        "INSERT INTO tiles (tile, kind, note, workspace, checkout, position)
         VALUES ('note-' || lower(hex(randomblob(8))), 'note', ?, ?, ?, (
             SELECT COALESCE(MAX(position), 0) + 1 FROM tiles
             WHERE workspace = ? AND checkout IS ?
         ))
         RETURNING tile",
        note,
        id,
        checkout,
        id,
        checkout
    )
    .fetch_one(pool)
    .await
    .map_err(missing)
}

pub async fn show_note(pool: &SqlitePool, tile: &str, note: Option<&str>) -> Result<(), Error> {
    let shown = sqlx::query!(
        "UPDATE tiles SET note = ? WHERE tile = ? AND kind = 'note'",
        note,
        tile
    )
    .execute(pool)
    .await?;
    if shown.rows_affected() == 0 {
        return Err(Error::Closed);
    }

    Ok(())
}

pub async fn open_monitor(
    pool: &SqlitePool,
    workspace: &str,
    checkout: Option<&str>,
) -> Result<String, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let mut tx = pool.begin().await?;
    let open = sqlx::query_scalar!(
        "SELECT tile FROM tiles WHERE kind = 'monitor' AND workspace = ? AND checkout IS ?",
        id,
        checkout
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(tile) = open {
        return Ok(tile);
    }

    let tile = sqlx::query_scalar!(
        "INSERT INTO tiles (tile, kind, workspace, checkout, position)
         VALUES ('monitor-' || lower(hex(randomblob(8))), 'monitor', ?, ?, (
             SELECT COALESCE(MAX(position), 0) + 1 FROM tiles
             WHERE workspace = ? AND checkout IS ?
         ))
         RETURNING tile",
        id,
        checkout,
        id,
        checkout
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(missing)?;
    tx.commit().await?;

    Ok(tile)
}

pub async fn close(pool: &SqlitePool, tile: &str) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM tiles WHERE tile = ? AND kind != 'terminal'",
        tile
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn close_in(pool: &SqlitePool, checkout: &Path) -> Result<bool, Error> {
    let mut tx = pool.begin().await?;
    let notes = sqlx::query!(
        "SELECT tile, checkout FROM tiles WHERE kind != 'terminal' AND checkout IS NOT NULL"
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut closed = false;
    for row in notes {
        if !row
            .checkout
            .is_some_and(|path| folders::same(Path::new(&path), checkout))
        {
            continue;
        }

        sqlx::query!("DELETE FROM tiles WHERE tile = ?", row.tile)
            .execute(&mut *tx)
            .await?;
        closed = true;
    }
    tx.commit().await?;

    Ok(closed)
}

pub async fn follow(pool: &SqlitePool, from: &Path, to: &Path) -> Result<bool, Error> {
    let mut tx = pool.begin().await?;
    let notes = sqlx::query!("SELECT tile, note FROM tiles WHERE note IS NOT NULL")
        .fetch_all(&mut *tx)
        .await?;

    let mut followed = false;
    for row in notes {
        let Some(rest) = row
            .note
            .as_deref()
            .and_then(|note| Path::new(note).strip_prefix(from).ok())
        else {
            continue;
        };

        let moved = if rest.as_os_str().is_empty() {
            to.to_path_buf()
        } else {
            to.join(rest)
        };
        let moved = moved.display().to_string();
        sqlx::query!("UPDATE tiles SET note = ? WHERE tile = ?", moved, row.tile)
            .execute(&mut *tx)
            .await?;
        followed = true;
    }
    tx.commit().await?;

    Ok(followed)
}

pub async fn arrange(pool: &SqlitePool, tile: &str, before: Option<&str>) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    let placed = sqlx::query!("SELECT workspace, checkout FROM tiles WHERE tile = ?", tile)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error::Closed)?;

    let mut order = sqlx::query_scalar!(
        "SELECT tile FROM tiles WHERE workspace = ? AND checkout IS ? ORDER BY position, id",
        placed.workspace,
        placed.checkout
    )
    .fetch_all(&mut *tx)
    .await?;
    order.retain(|id| id != tile);
    let at = before
        .and_then(|before| order.iter().position(|id| id == before))
        .unwrap_or(order.len());
    order.insert(at, tile.to_owned());

    for (position, id) in (0_i64..).zip(&order) {
        sqlx::query!("UPDATE tiles SET position = ? WHERE tile = ?", position, id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    Ok(())
}

fn missing(error: sqlx::Error) -> Error {
    match error {
        sqlx::Error::Database(error) if error.is_foreign_key_violation() => Error::Missing,
        error => error.into(),
    }
}

#[cfg(test)]
mod tests;
