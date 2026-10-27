use std::path::Path;

use sqlx::SqlitePool;

use super::Error;
use crate::folders;

pub async fn members(pool: &SqlitePool, workspace: &str) -> Result<Vec<String>, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;

    Ok(
        sqlx::query_scalar!("SELECT tile FROM tiles WHERE workspace = ?", id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn place(
    pool: &SqlitePool,
    terminal: &str,
    workspace: &str,
    checkout: Option<&str>,
) -> Result<(), Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    sqlx::query!(
        "INSERT INTO tiles (tile, workspace, checkout, position)
         VALUES (?, ?, ?, (
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

pub async fn terminals_in(pool: &SqlitePool, checkout: &Path) -> Result<Vec<String>, Error> {
    let rows = sqlx::query!("SELECT tile, checkout FROM tiles WHERE checkout IS NOT NULL")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .filter(|row| {
            row.checkout
                .as_deref()
                .is_some_and(|path| folders::same(Path::new(path), checkout))
        })
        .map(|row| row.tile)
        .collect())
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
