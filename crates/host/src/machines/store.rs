use protocol::machines::MachineId;
use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the machines: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("that machine is no longer set up")]
    Missing,
    #[error("one machine has to stay connected")]
    Last,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    pub id: String,
    pub target: String,
    pub triple: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    pub server: Option<Server>,
    pub name: Option<String>,
    pub connected: bool,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Machine>, Error> {
    Ok(sqlx::query!(
        r#"SELECT host, target, triple, name, connected AS "connected: bool"
        FROM machines ORDER BY host IS NOT NULL, id"#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| Machine {
        server: match (row.host, row.target, row.triple) {
            (Some(id), Some(target), Some(triple)) => Some(Server { id, target, triple }),
            _ => None,
        },
        name: row.name,
        connected: row.connected,
    })
    .collect())
}

pub async fn server(pool: &SqlitePool, id: &str) -> Result<Option<Machine>, Error> {
    Ok(list(pool).await?.into_iter().find(|machine| {
        machine
            .server
            .as_ref()
            .is_some_and(|server| server.id == id)
    }))
}

pub async fn add_computer(pool: &SqlitePool) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!("INSERT OR IGNORE INTO machines (connected) VALUES (TRUE)")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("UPDATE machines SET connected = TRUE WHERE host IS NULL")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(())
}

pub async fn add_server(pool: &SqlitePool, server: &Server) -> Result<(), Error> {
    let mut tx = pool.begin().await?;
    let name = sqlx::query_scalar!("SELECT name FROM machines WHERE host = ?", server.id)
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
    sqlx::query!(
        "DELETE FROM machines WHERE host = ? OR target = ?",
        server.id,
        server.target
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO machines (host, target, triple, name) VALUES (?, ?, ?, ?)",
        server.id,
        server.target,
        server.triple,
        name
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(())
}

pub async fn rename(pool: &SqlitePool, machine: &MachineId, name: &str) -> Result<(), Error> {
    let host = host(machine);
    let name = Some(name.trim()).filter(|name| !name.is_empty());
    let changed = sqlx::query!("UPDATE machines SET name = ? WHERE host IS ?", name, host)
        .execute(pool)
        .await?
        .rows_affected();
    if changed == 0 {
        return Err(Error::Missing);
    }

    Ok(())
}

pub async fn set_connected(
    pool: &SqlitePool,
    machine: &MachineId,
    connected: bool,
) -> Result<(), Error> {
    let host = host(machine);
    let mut tx = pool.begin().await?;
    let changed = sqlx::query!(
        "UPDATE machines SET connected = ? WHERE host IS ?",
        connected,
        host
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if changed == 0 {
        return Err(Error::Missing);
    }

    keep_one(tx).await
}

pub async fn remove(pool: &SqlitePool, machine: &MachineId) -> Result<(), Error> {
    let host = host(machine);
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM machines WHERE host IS ?", host)
        .execute(&mut *tx)
        .await?;

    keep_one(tx).await
}

fn host(machine: &MachineId) -> Option<&str> {
    match machine {
        MachineId::Computer => None,
        MachineId::Server { id } => Some(id),
    }
}

async fn keep_one(mut tx: Transaction<'_, Sqlite>) -> Result<(), Error> {
    let connected = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM machines WHERE connected) AS "connected: bool""#
    )
    .fetch_one(&mut *tx)
    .await?;
    if !connected {
        return Err(Error::Last);
    }

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests;
