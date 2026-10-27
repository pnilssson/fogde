mod identity;
mod prune;
mod store;
mod tiles;
mod watch;

pub use identity::Searches;
pub use tiles::{close_terminals_in, follow};
pub use watch::{Watch, changes};

use std::path::{Path, PathBuf};

use futures_util::future::join_all;
use protocol::rpc::Failure;
use protocol::terminals::Terminal;
use protocol::workspaces::{
    Checkout, CreateWorkspace, DeleteWorkspace, ListWorkspaces, MonitorTile, NoteTile,
    PlaceTerminal, PlaceTerminalParams, Tile, Workspace,
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
    tiles::register(router);
}

pub async fn place(
    session: &Session,
    terminal: &str,
    workspace: &str,
    checkout: Option<&str>,
) -> Result<(), Failure> {
    store::place(pool(session).await?, terminal, workspace, checkout)
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

pub fn worktree_of(folder: &Path, checkouts: &[Checkout]) -> Option<String> {
    checkouts
        .iter()
        .filter(|checkout| folders::inside(folder, Path::new(&checkout.path)))
        .max_by_key(|checkout| checkout.path.len())
        .filter(|checkout| !checkout.main)
        .map(|checkout| checkout.path.clone())
}

impl Handler for ListWorkspaces {
    async fn handle(session: &Session, (): ()) -> Result<Vec<Workspace>, Failure> {
        let placing = session.shared.placing.lock().await;
        let terminals = crate::terminals::list(session).await?;
        let pool = pool(session).await?;
        let listing = store::list(pool, &terminals).await.map_err(failure)?;
        drop(placing);

        if let Err(error) = crate::terminals::close(session, &listing.loose).await {
            tracing::warn!(?error, "could not close terminals without a workspace");
        }

        Ok(
            join_all(listing.workspaces.into_iter().map(|workspace| async {
                let (workspace, missing) = locate(session, pool, workspace).await;
                let checkouts = if missing {
                    Vec::new()
                } else {
                    checkouts(Path::new(&workspace.folder)).await
                };
                let workspace = prune::prune(session, pool, workspace, &checkouts).await;

                compose(workspace, missing, checkouts)
            }))
            .await,
        )
    }
}

impl Handler for CreateWorkspace {
    async fn handle(session: &Session, folder: String) -> Result<Workspace, Failure> {
        let folder = folders::absolute(&folder).map_err(failure)?;
        folders::directory(&folder).await.map_err(failure)?;

        create(session, &folder).await
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
        place(
            session,
            &params.terminal,
            &params.workspace,
            params.checkout.as_deref(),
        )
        .await
    }
}

pub async fn create(session: &Session, folder: &Path) -> Result<Workspace, Failure> {
    let identity = identity::of(folder).await;
    let stored = store::create(
        pool(session).await?,
        &folder.display().to_string(),
        identity.as_deref(),
    )
    .await
    .map_err(failure)?;

    Ok(compose(stored, false, checkouts(folder).await))
}

async fn locate(
    session: &Session,
    pool: &SqlitePool,
    mut workspace: store::Workspace,
) -> (store::Workspace, bool) {
    let folder = PathBuf::from(&workspace.folder);
    if folders::directory(&folder).await.is_ok() {
        let identity = identity::of(&folder).await;
        if identity.is_some() && identity != workspace.identity {
            workspace.identity = identity;
            save(pool, &workspace).await;
        }

        return (workspace, false);
    }

    let moved = match &workspace.identity {
        Some(identity) => {
            session
                .shared
                .searches
                .find(&workspace.id, &folder, identity)
                .await
        }
        None => None,
    };
    let Some(moved) = moved else {
        return (workspace, true);
    };

    workspace.folder = moved.display().to_string();
    save(pool, &workspace).await;

    (workspace, false)
}

async fn save(pool: &SqlitePool, workspace: &store::Workspace) {
    let saved = store::relocate(
        pool,
        &workspace.id,
        &workspace.folder,
        workspace.identity.as_deref(),
    )
    .await;
    if let Err(error) = saved {
        tracing::warn!(?error, workspace = %workspace.id, "could not save the workspace folder");
    }
}

fn compose(workspace: store::Workspace, missing: bool, checkouts: Vec<Checkout>) -> Workspace {
    let folder = PathBuf::from(&workspace.folder);
    let tiles = workspace
        .tiles
        .into_iter()
        .map(|tile| match tile {
            Tile::Terminal(terminal) => Tile::Terminal(Terminal {
                checkout: terminal.checkout.and_then(|path| listed(&path, &checkouts)),
                ..terminal
            }),
            Tile::Note(note) => Tile::Note(NoteTile {
                checkout: note.checkout.and_then(|path| listed(&path, &checkouts)),
                ..note
            }),
            Tile::Monitor(monitor) => Tile::Monitor(MonitorTile {
                checkout: monitor.checkout.and_then(|path| listed(&path, &checkouts)),
                ..monitor
            }),
        })
        .collect();

    Workspace {
        id: workspace.id,
        name: folder_name(&folder),
        folder: checkouts
            .iter()
            .find(|checkout| folders::same(&folder, Path::new(&checkout.path)))
            .map_or_else(
                || folder.display().to_string(),
                |checkout| checkout.path.clone(),
            ),
        missing,
        checkouts,
        tiles,
    }
}

fn listed(path: &str, checkouts: &[Checkout]) -> Option<String> {
    checkouts
        .iter()
        .find(|checkout| {
            !checkout.main && folders::same(Path::new(path), Path::new(&checkout.path))
        })
        .map(|checkout| checkout.path.clone())
}

fn folder_name(folder: &Path) -> String {
    folder.file_name().map_or_else(
        || folder.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
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
    fn finds_the_linked_worktree_of_a_folder() {
        let root = std::env::temp_dir();
        let repo = root.join("app");
        let nested = repo.join("worktrees").join("login");
        let checkouts = [checkout(&repo, true), checkout(&nested, false)];

        assert_eq!(worktree_of(&repo.join("crates"), &checkouts), None);
        assert_eq!(
            worktree_of(&nested.join("src"), &checkouts),
            Some(nested.display().to_string())
        );
        assert_eq!(worktree_of(&root.join("app-old"), &checkouts), None);
    }

    #[test]
    fn a_stored_worktree_takes_the_spelling_git_lists() {
        let repo = std::env::temp_dir().join("app");
        let login = repo.join("worktrees").join("login");
        let checkouts = [checkout(&repo, true), checkout(&login, false)];
        let stored = format!("{}{}", login.display(), std::path::MAIN_SEPARATOR);

        assert_eq!(
            listed(&stored, &checkouts),
            Some(login.display().to_string())
        );
        assert_eq!(listed(&repo.display().to_string(), &checkouts), None);
    }
}
