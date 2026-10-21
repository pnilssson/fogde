mod store;

use std::path::{Path, PathBuf};

use futures_util::future::join_all;
use protocol::rpc::Failure;
use protocol::terminals::Terminal;
use protocol::workspaces::{
    Checkout, CreateWorkspace, DeleteWorkspace, ListWorkspaces, PlaceTerminal, PlaceTerminalParams,
    RenameWorkspace, RenameWorkspaceParams, Workspace,
};
use sqlx::SqlitePool;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<ListWorkspaces>();
    router.add::<CreateWorkspace>();
    router.add::<RenameWorkspace>();
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
        let listing = store::list(pool(session).await?, &terminals)
            .await
            .map_err(failure)?;
        drop(placing);

        if let Err(error) = crate::terminals::close(session, &listing.loose).await {
            tracing::warn!(?error, "could not close terminals without a workspace");
        }

        Ok(join_all(listing.workspaces.into_iter().map(compose)).await)
    }
}

impl Handler for CreateWorkspace {
    async fn handle(session: &Session, folder: String) -> Result<Workspace, Failure> {
        let folder = folders::absolute(&folder).map_err(failure)?;
        folders::directory(&folder).await.map_err(failure)?;

        create(session, &folder).await
    }
}

impl Handler for RenameWorkspace {
    async fn handle(session: &Session, params: RenameWorkspaceParams) -> Result<(), Failure> {
        store::rename(pool(session).await?, &params.workspace, &params.name)
            .await
            .map_err(failure)
    }
}

impl Handler for DeleteWorkspace {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        let _placing = session.shared.placing.lock().await;
        let pool = pool(session).await?;
        let terminals = store::members(pool, &id).await.map_err(failure)?;
        crate::terminals::close(session, &terminals).await?;

        store::delete(pool, &id).await.map_err(failure)
    }
}

impl Handler for PlaceTerminal {
    async fn handle(session: &Session, params: PlaceTerminalParams) -> Result<(), Failure> {
        place(session, &params.terminal, &params.workspace).await
    }
}

pub async fn create(session: &Session, folder: &Path) -> Result<Workspace, Failure> {
    let stored = store::create(pool(session).await?, &folder.display().to_string())
        .await
        .map_err(failure)?;

    Ok(compose(stored).await)
}

async fn compose(workspace: store::Workspace) -> Workspace {
    let folder = PathBuf::from(&workspace.folder);
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
        let repo = root.join("app");
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
        assert_eq!(checkout_of(&root.join("app-old"), &checkouts), None);
    }
}
