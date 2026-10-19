use protocol::terminals::Terminal;
use sqlx::{SqliteConnection, SqlitePool};

const HOME: &str = "Home";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not read the workspaces: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("a workspace needs a name")]
    Name,
    #[error("that workspace no longer exists")]
    Missing,
    #[error("the Home workspace can't be deleted")]
    Home,
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder: Option<String>,
    pub terminals: Vec<Terminal>,
}

pub async fn list(pool: &SqlitePool, terminals: &[Terminal]) -> Result<Vec<Workspace>, Error> {
    let mut tx = pool.begin().await?;
    let home = home(&mut tx).await?;

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
    for terminal in terminals
        .iter()
        .filter(|terminal| !known.contains(&terminal.id))
    {
        sqlx::query!(
            "INSERT INTO members (terminal, workspace) VALUES (?, ?)",
            terminal.id,
            home
        )
        .execute(&mut *tx)
        .await?;
    }

    let mut workspaces: Vec<Workspace> =
        sqlx::query!("SELECT id, name, folder FROM workspaces ORDER BY id")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|row| Workspace {
                id: row.id.to_string(),
                name: row.name,
                folder: row.folder,
                terminals: Vec::new(),
            })
            .collect();
    let members = sqlx::query!("SELECT workspace, terminal FROM members ORDER BY id")
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;

    for member in members {
        let id = member.workspace.to_string();
        let terminal = terminals
            .iter()
            .find(|terminal| terminal.id == member.terminal);
        let workspace = workspaces.iter_mut().find(|workspace| workspace.id == id);
        if let (Some(terminal), Some(workspace)) = (terminal, workspace) {
            workspace.terminals.push(terminal.clone());
        }
    }

    Ok(workspaces)
}

pub async fn create(pool: &SqlitePool, name: &str, folder: &str) -> Result<Workspace, Error> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Name);
    }

    let mut tx = pool.begin().await?;
    home(&mut tx).await?;
    let id = sqlx::query!(
        "INSERT INTO workspaces (name, folder) VALUES (?, ?)",
        name,
        folder
    )
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    tx.commit().await?;

    Ok(Workspace {
        id: id.to_string(),
        name: name.to_owned(),
        folder: Some(folder.to_owned()),
        terminals: Vec::new(),
    })
}

pub async fn delete(pool: &SqlitePool, workspace: &str) -> Result<Vec<String>, Error> {
    let id: i64 = workspace.parse().map_err(|_| Error::Missing)?;
    let mut tx = pool.begin().await?;
    if home(&mut tx).await? == id {
        return Err(Error::Home);
    }

    let terminals = sqlx::query_scalar!("SELECT terminal FROM members WHERE workspace = ?", id)
        .fetch_all(&mut *tx)
        .await?;
    let deleted = sqlx::query!("DELETE FROM workspaces WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(Error::Missing);
    }
    tx.commit().await?;

    Ok(terminals)
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

async fn home(db: &mut SqliteConnection) -> Result<i64, Error> {
    let first = sqlx::query_scalar!("SELECT id FROM workspaces ORDER BY id LIMIT 1")
        .fetch_optional(&mut *db)
        .await?;
    if let Some(id) = first {
        return Ok(id);
    }

    Ok(
        sqlx::query!("INSERT INTO workspaces (name) VALUES (?)", HOME)
            .execute(&mut *db)
            .await?
            .last_insert_rowid(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn terminals(ids: &[&str]) -> Vec<Terminal> {
        ids.iter()
            .map(|&id| Terminal {
                id: id.to_owned(),
                name: format!("Terminal {id}"),
                folder: "/home/me".to_owned(),
                checkout: None,
            })
            .collect()
    }

    fn ids(workspace: &Workspace) -> Vec<&str> {
        workspace
            .terminals
            .iter()
            .map(|terminal| terminal.id.as_str())
            .collect()
    }

    #[tokio::test]
    async fn creates_one_home_workspace_without_a_folder() {
        let pool = db::memory().await;

        list(&pool, &[]).await.unwrap();
        let workspaces = list(&pool, &[]).await.unwrap();

        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, HOME);
        assert_eq!(workspaces[0].folder, None);
    }

    #[tokio::test]
    async fn loose_terminals_join_home_and_closed_ones_leave() {
        let pool = db::memory().await;
        list(&pool, &terminals(&["w1", "w2"])).await.unwrap();

        let workspaces = list(&pool, &terminals(&["w2", "w3"])).await.unwrap();

        assert_eq!(ids(&workspaces[0]), ["w2", "w3"]);
        assert_eq!(workspaces[0].terminals[0].name, "Terminal w2");
    }

    #[tokio::test]
    async fn placed_terminals_stay_in_their_workspace() {
        let pool = db::memory().await;
        let work = create(&pool, " Work ", "/repos/work").await.unwrap();
        list(&pool, &terminals(&["w1", "w2"])).await.unwrap();

        place(&pool, "w1", &work.id).await.unwrap();
        let workspaces = list(&pool, &terminals(&["w1", "w2"])).await.unwrap();

        assert_eq!(work.name, "Work");
        assert_eq!(workspaces[1].folder.as_deref(), Some("/repos/work"));
        assert_eq!(ids(&workspaces[0]), ["w2"]);
        assert_eq!(ids(&workspaces[1]), ["w1"]);
    }

    #[tokio::test]
    async fn new_terminals_can_be_placed_before_they_are_listed() {
        let pool = db::memory().await;
        let work = create(&pool, "Work", "/repos/work").await.unwrap();

        place(&pool, "w1", &work.id).await.unwrap();
        let workspaces = list(&pool, &terminals(&["w1"])).await.unwrap();

        assert_eq!(ids(&workspaces[0]), Vec::<&str>::new());
        assert_eq!(ids(&workspaces[1]), ["w1"]);
    }

    #[tokio::test]
    async fn deleting_a_workspace_hands_back_its_terminals() {
        let pool = db::memory().await;
        let work = create(&pool, "Work", "/repos/work").await.unwrap();
        list(&pool, &terminals(&["w1", "w2"])).await.unwrap();
        place(&pool, "w1", &work.id).await.unwrap();

        let closing = delete(&pool, &work.id).await.unwrap();
        let workspaces = list(&pool, &terminals(&["w1", "w2"])).await.unwrap();

        assert_eq!(closing, ["w1"]);
        assert_eq!(workspaces.len(), 1);
        assert_eq!(ids(&workspaces[0]), ["w2", "w1"]);
    }

    #[tokio::test]
    async fn keeps_home_and_rejects_unknown_workspaces_on_delete() {
        let pool = db::memory().await;
        let home = list(&pool, &[]).await.unwrap().remove(0);

        assert!(matches!(delete(&pool, &home.id).await, Err(Error::Home)));
        assert!(matches!(delete(&pool, "9").await, Err(Error::Missing)));
        assert!(matches!(delete(&pool, "home").await, Err(Error::Missing)));
        assert_eq!(list(&pool, &[]).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rejects_blank_names_and_unknown_workspaces() {
        let pool = db::memory().await;

        assert!(matches!(
            create(&pool, "  ", "/repos/work").await,
            Err(Error::Name)
        ));
        assert!(matches!(place(&pool, "w1", "9").await, Err(Error::Missing)));
        assert!(matches!(
            place(&pool, "w1", "home").await,
            Err(Error::Missing)
        ));
    }
}
