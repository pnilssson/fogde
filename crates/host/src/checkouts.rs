use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use futures_util::future::join_all;
use protocol::checkouts::{
    Branch, ChangeKind, CheckoutStatus, CleanUp, CleanUpParams, Commit, CommitParams, CreateBranch,
    Diff, DiffParams, DiffScope, Fetch, FileChange, FileDiff, GetCheckouts, GetDiff, LastCommit,
    ListBranches, Pull, Push, Switch, SwitchParams, Undo, UndoParams,
};
use protocol::repos::Repo;
use protocol::rpc::Failure;
use tokio::fs;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

const FILES: usize = 500;

pub fn register(router: &mut Router) {
    router.add::<GetCheckouts>();
    router.add::<ListBranches>();
    router.add::<Commit>();
    router.add::<Push>();
    router.add::<Pull>();
    router.add::<Switch>();
    router.add::<CreateBranch>();
    router.add::<Fetch>();
    router.add::<Undo>();
    router.add::<CleanUp>();
    router.add::<GetDiff>();
}

impl Handler for GetCheckouts {
    async fn handle(
        session: &Session,
        folders: Vec<String>,
    ) -> Result<Vec<CheckoutStatus>, Failure> {
        let hosts = session.shared.hosts();
        let found = join_all(folders.iter().map(|folder| async move {
            let path = folders::absolute(folder).ok()?;
            git::root(&path).await
        }))
        .await;
        let mut roots: Vec<PathBuf> = Vec::new();
        for root in found.into_iter().flatten() {
            if !roots.contains(&root) {
                roots.push(root);
            }
        }

        let checked = join_all(roots.iter().map(|root| checkout(root, &hosts))).await;

        Ok(checked.into_iter().flatten().collect())
    }
}

pub struct Published {
    pub repo: Option<Repo>,
    pub owner: Option<String>,
    pub pull: Option<u32>,
}

pub async fn published(root: &Path, branch: Option<&str>, hosts: &[String]) -> Published {
    let config = git::config(root, branch).await;
    let remotes = github::signed(config.remotes, hosts);

    Published {
        repo: github::base(&remotes),
        owner: github::pushed(&remotes, config.push.as_deref()).map(|repo| repo.owner),
        pull: github::pulled(
            &remotes,
            config.tracking.as_deref(),
            config.merge.as_deref(),
        ),
    }
}

async fn checkout(root: &Path, hosts: &[String]) -> Option<CheckoutStatus> {
    let read = async {
        let status = git::status(root).await?;
        let published = published(root, status.branch.as_deref(), hosts).await;

        Ok::<_, git::Error>((status, published))
    };
    let (read, fetched, last) = tokio::join!(read, git::fetched(root), git::last(root));
    let (status, published) = match read {
        Ok(read) => read,
        Err(error) => {
            tracing::debug!(%error, "could not read a checkout");
            return None;
        }
    };

    Some(CheckoutStatus {
        path: root.display().to_string(),
        repo: published.repo,
        owner: published.owner,
        pull: published.pull,
        truncated: status.files.len() > FILES,
        branch: status.branch,
        upstream: status.upstream,
        ahead: status.ahead,
        behind: status.behind,
        files: status.files.into_iter().take(FILES).map(file).collect(),
        fetched: fetched
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .and_then(|since| u64::try_from(since.as_millis()).ok()),
        last: last.map(|last| LastCommit {
            sha: last.sha,
            subject: last.subject,
            date: last.date,
            undoable: last.undoable,
        }),
    })
}

fn file(file: git::File) -> FileChange {
    FileChange {
        path: file.path,
        from: file.from,
        change: kind(file.change),
    }
}

fn kind(change: git::Change) -> ChangeKind {
    match change {
        git::Change::Added => ChangeKind::Added,
        git::Change::Modified => ChangeKind::Modified,
        git::Change::Deleted => ChangeKind::Deleted,
        git::Change::Renamed => ChangeKind::Renamed,
        git::Change::Copied => ChangeKind::Copied,
        git::Change::Untracked => ChangeKind::Untracked,
        git::Change::Conflicted => ChangeKind::Conflicted,
    }
}

impl Handler for ListBranches {
    async fn handle(_: &Session, path: String) -> Result<Vec<Branch>, Failure> {
        let branches = git::branches(&root(&path).await?).await.map_err(failure)?;

        Ok(branches
            .into_iter()
            .map(|branch| Branch {
                name: branch.name,
                worktree: branch.worktree,
                remote: branch.remote,
                date: branch.date,
            })
            .collect())
    }
}

impl Handler for Commit {
    async fn handle(_: &Session, params: CommitParams) -> Result<(), Failure> {
        let root = root(&params.path).await?;

        git::commit(&root, &params.paths, &params.message)
            .await
            .map_err(failure)
    }
}

impl Handler for Push {
    async fn handle(_: &Session, path: String) -> Result<(), Failure> {
        git::push(&root(&path).await?).await.map_err(failure)
    }
}

impl Handler for Pull {
    async fn handle(_: &Session, path: String) -> Result<(), Failure> {
        git::pull(&root(&path).await?).await.map_err(failure)
    }
}

impl Handler for Switch {
    async fn handle(_: &Session, params: SwitchParams) -> Result<(), Failure> {
        let root = root(&params.path).await?;

        git::switch(&root, &params.branch).await.map_err(failure)
    }
}

impl Handler for CreateBranch {
    async fn handle(_: &Session, params: SwitchParams) -> Result<(), Failure> {
        let root = root(&params.path).await?;

        git::branch_off(&root, &params.branch)
            .await
            .map_err(failure)
    }
}

impl Handler for Fetch {
    async fn handle(_: &Session, path: String) -> Result<(), Failure> {
        git::refresh(&root(&path).await?).await.map_err(failure)
    }
}

impl Handler for GetDiff {
    async fn handle(_: &Session, params: DiffParams) -> Result<Diff, Failure> {
        let root = root(&params.path).await?;
        let scope = match params.scope {
            DiffScope::Uncommitted => git::Scope::Uncommitted,
            DiffScope::Branch => git::Scope::Branch,
        };
        let diff = git::diff(&root, scope).await.map_err(failure)?;

        Ok(Diff {
            base: diff.base,
            truncated: diff.truncated,
            files: diff
                .files
                .into_iter()
                .map(|file| FileDiff {
                    path: file.path,
                    from: file.from,
                    change: kind(file.change),
                    added: file.added,
                    removed: file.removed,
                    patch: file.patch,
                    binary: file.binary,
                    large: file.large,
                })
                .collect(),
        })
    }
}

impl Handler for Undo {
    async fn handle(_: &Session, params: UndoParams) -> Result<(), Failure> {
        let root = root(&params.path).await?;

        git::undo(&root, &params.sha).await.map_err(failure)
    }
}

impl Handler for CleanUp {
    async fn handle(session: &Session, params: CleanUpParams) -> Result<(), Failure> {
        let root = root(&params.path).await?;
        let branch = git::landed(&root, &params.sha, &params.base)
            .await
            .map_err(failure)?;
        let main = crate::workspaces::checkouts(&root)
            .await
            .into_iter()
            .find(|checkout| checkout.main)
            .map_or_else(|| root.clone(), |checkout| PathBuf::from(checkout.path));
        if same(&main, &root).await {
            return git::leave(&root, &branch, &params.base)
                .await
                .map_err(failure);
        }

        crate::workspaces::close_terminals_in(session, &root).await?;
        git::remove_worktree(&main, &root).await.map_err(failure)?;

        git::delete_branch(&main, &branch).await.map_err(failure)
    }
}

async fn same(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a).await, fs::canonicalize(b).await) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

pub async fn root(path: &str) -> Result<PathBuf, Failure> {
    let path = folders::absolute(path).map_err(failure)?;

    git::root(&path)
        .await
        .ok_or_else(|| failure(format!("{} is not a Git repository", path.display())))
}
