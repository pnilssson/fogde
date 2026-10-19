mod store;

use std::path::{Path, PathBuf};

use futures_util::future::join_all;
use protocol::rpc::Failure;
use protocol::terminals::Terminal;
use protocol::workspaces::{
    Checkout, CreateWorkspace, CreateWorkspaceParams, DeleteWorkspace, ListWorkspaces,
    PlaceTerminal, PlaceTerminalParams, Workspace,
};
use sqlx::SqlitePool;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<ListWorkspaces>();
    router.add::<CreateWorkspace>();
    router.add::<DeleteWorkspace>();
    router.add::<PlaceTerminal>();
}

pub async fn place(session: &Session, terminal: &str, workspace: &str) -> Result<(), Failure> {
    store::place(pool(session).await?, terminal, workspace)
        .await
        .map_err(failure)
}

pub async fn checkouts(folder: &Path) -> Vec<Checkout> {
    git::worktrees(folder)
        .await
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, worktree)| Checkout {
            path: worktree.path.display().to_string(),
            branch: worktree.branch,
            main: index == 0,
        })
        .collect()
}

pub fn checkout_of(folder: &Path, checkouts: &[Checkout]) -> Option<String> {
    checkouts
        .iter()
        .filter(|checkout| folders::inside(folder, Path::new(&checkout.path)))
        .max_by_key(|checkout| checkout.path.len())
        .map(|checkout| checkout.path.clone())
}

impl Handler for ListWorkspaces {
    async fn handle(session: &Session, (): ()) -> Result<Vec<Workspace>, Failure> {
        let placing = session.shared.placing.lock().await;
        let terminals = crate::terminals::list(session).await?;
        let stored = store::list(pool(session).await?, &terminals)
            .await
            .map_err(failure)?;
        drop(placing);
        let home = folders::home().map_err(failure)?;

        Ok(join_all(
            stored
                .into_iter()
                .map(|workspace| compose(workspace, &home)),
        )
        .await)
    }
}

impl Handler for CreateWorkspace {
    async fn handle(
        session: &Session,
        params: CreateWorkspaceParams,
    ) -> Result<Workspace, Failure> {
        let folder = folders::absolute(&params.folder).map_err(failure)?;
        folders::directory(&folder).await.map_err(failure)?;
        let home = folders::home().map_err(failure)?;

        let stored = store::create(
            pool(session).await?,
            &params.name,
            &folder.display().to_string(),
        )
        .await
        .map_err(failure)?;

        Ok(compose(stored, &home).await)
    }
}

impl Handler for DeleteWorkspace {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        let _placing = session.shared.placing.lock().await;
        let terminals = store::delete(pool(session).await?, &id)
            .await
            .map_err(failure)?;

        crate::terminals::close(session, &terminals).await
    }
}

impl Handler for PlaceTerminal {
    async fn handle(session: &Session, params: PlaceTerminalParams) -> Result<(), Failure> {
        place(session, &params.terminal, &params.workspace).await
    }
}

async fn compose(workspace: store::Workspace, home: &Path) -> Workspace {
    let Some(folder) = workspace.folder.map(PathBuf::from) else {
        return Workspace {
            id: workspace.id,
            name: workspace.name,
            folder: home.display().to_string(),
            checkouts: Vec::new(),
            terminals: workspace.terminals,
        };
    };
    let checkouts = checkouts(&folder).await;
    let terminals = workspace
        .terminals
        .into_iter()
        .map(|terminal| Terminal {
            checkout: checkout_of(Path::new(&terminal.folder), &checkouts),
            ..terminal
        })
        .collect();

    Workspace {
        id: workspace.id,
        name: workspace.name,
        folder: checkouts
            .iter()
            .find(|checkout| folders::same(&folder, Path::new(&checkout.path)))
            .map_or_else(
                || folder.display().to_string(),
                |checkout| checkout.path.clone(),
            ),
        checkouts,
        terminals,
    }
}

async fn pool(session: &Session) -> Result<&SqlitePool, Failure> {
    session.shared.db.pool().await.map_err(failure)
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

    #[test]
    fn finds_the_innermost_checkout_of_a_folder() {
        let root = std::env::temp_dir();
        let repo = root.join("fogde");
        let nested = repo.join("worktrees").join("login");
        let checkouts = [checkout(&repo, true), checkout(&nested, false)];

        assert_eq!(
            checkout_of(&repo.join("crates"), &checkouts),
            Some(repo.display().to_string())
        );
        assert_eq!(
            checkout_of(&nested.join("src"), &checkouts),
            Some(nested.display().to_string())
        );
        assert_eq!(checkout_of(&root.join("fogde-old"), &checkouts), None);
    }
}
