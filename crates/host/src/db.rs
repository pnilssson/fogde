use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use tokio::fs;
use tokio::sync::OnceCell;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not open the database: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not open the database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("could not update the database: {0}")]
    Migrate(#[from] MigrateError),
}

pub struct Db {
    path: PathBuf,
    pool: OnceCell<SqlitePool>,
}

impl Db {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            pool: OnceCell::new(),
        }
    }

    pub async fn pool(&self) -> Result<&SqlitePool, Error> {
        self.pool.get_or_try_init(|| open(&self.path)).await
    }
}

async fn open(path: &Path) -> Result<SqlitePool, Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(options).await?;
    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
pub async fn memory() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(
            SqliteConnectOptions::new()
                .in_memory(true)
                .foreign_keys(true),
        )
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();

    pool
}
