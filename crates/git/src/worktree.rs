use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::fs;

use crate::Error;
use crate::base::{fetch, mainline};
use crate::dirs::Dirs;
use crate::history::unfinished;
use crate::include;
use crate::run::git;
use list::parse;

mod list;

const RELEASE_TRIES: u32 = 20;
const RELEASE_WAIT: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub main: bool,
}

pub async fn worktrees(repo: &Path) -> Result<Vec<Worktree>, Error> {
    let listed = git(repo, ["worktree", "list", "--porcelain"]).await?;

    Ok(parse(&String::from_utf8_lossy(&listed)))
}

pub async fn add_worktree(repo: &Path, path: &Path, branch: &str) -> Result<(), Error> {
    let reference = format!("refs/heads/{branch}");
    if git(repo, ["show-ref", "--verify", "--quiet", &reference])
        .await
        .is_ok()
    {
        return Err(Error::Git(format!(
            "a branch named {branch} already exists; choose another name"
        )));
    }

    let base = base(repo).await;
    git(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("--no-track"),
            OsStr::new("-b"),
            OsStr::new(branch),
            path.as_os_str(),
            OsStr::new(&base),
        ],
    )
    .await?;

    let Err(error) = include::copy(repo, path).await else {
        return Ok(());
    };
    undo(repo, path, Some(branch)).await;

    Err(error)
}

pub async fn add_detached(repo: &Path, path: &Path) -> Result<(), Error> {
    git(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("--detach"),
            path.as_os_str(),
            OsStr::new("HEAD"),
        ],
    )
    .await?;

    let Err(error) = include::copy(repo, path).await else {
        return Ok(());
    };
    undo(repo, path, None).await;

    Err(error)
}

pub async fn discard_worktree(repo: &Path, path: &Path) {
    undo(repo, path, None).await;
}

pub async fn removable(repo: &Path, path: &Path) -> Result<(), Error> {
    if !linked(repo, path).await? {
        return Err(Error::Git(format!(
            "{} is not a worktree of this repository",
            path.display()
        )));
    }
    if let Some(operation) = unfinished(path).await? {
        return Err(Error::Git(format!(
            "finish or abort the {operation} in a terminal first"
        )));
    }
    if !clean(path).await? {
        return Err(Error::Git("commit or discard its changes first".to_owned()));
    }

    let dirs = Dirs::of(path).await?;
    if fs::try_exists(dirs.path("locked")).await? {
        return Err(Error::Git(
            "the worktree is locked; unlock it in a terminal first".to_owned(),
        ));
    }
    if fs::try_exists(dirs.path("modules")).await? || submodules(path).await? {
        return Err(Error::Git(
            "remove a worktree with submodules in a terminal".to_owned(),
        ));
    }
    if loose(path).await? {
        return Err(Error::Git(
            "its commits are on no branch; create a branch for them first".to_owned(),
        ));
    }

    Ok(())
}

pub async fn remove_worktree(repo: &Path, path: &Path) -> Result<Option<String>, Error> {
    removable(repo, path).await?;
    let branch = git(path, ["symbolic-ref", "--quiet", "--short", "HEAD"])
        .await
        .ok()
        .map(|branch| String::from_utf8_lossy(&branch).trim().to_owned());

    let removed = git(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("remove"),
            path.as_os_str(),
        ],
    )
    .await;
    if let Err(error) = removed
        && linked(repo, path).await.unwrap_or(true)
    {
        return Err(error);
    }
    release(path).await?;

    Ok(branch)
}

pub async fn clean(path: &Path) -> Result<bool, Error> {
    let status = git(path, ["status", "--porcelain", "--ignore-submodules=none"]).await?;

    Ok(status.is_empty())
}

pub async fn remove_merged_branch(repo: &Path, branch: &str) -> Result<bool, Error> {
    let tip = format!("refs/heads/{branch}");
    let mut bases = vec!["HEAD".to_owned()];
    bases.extend(mainline(repo).await.map(|mainline| mainline.reference));
    let mut merged = false;
    for base in bases {
        if git(repo, ["merge-base", "--is-ancestor", &tip, &base])
            .await
            .is_ok()
        {
            merged = true;
            break;
        }
    }
    if !merged {
        return Ok(false);
    }

    git(repo, ["branch", "--delete", "--force", branch]).await?;

    Ok(true)
}

async fn base(repo: &Path) -> String {
    if let Err(error) = fetch(repo).await {
        tracing::warn!(%error, "could not fetch before adding a worktree");
    }

    mainline(repo)
        .await
        .map_or_else(|| "HEAD".to_owned(), |mainline| mainline.reference)
}

async fn submodules(path: &Path) -> Result<bool, Error> {
    let staged = git(path, ["ls-files", "--stage", "-z"]).await?;
    for entry in String::from_utf8_lossy(&staged).split('\0') {
        let Some((meta, name)) = entry.split_once('\t') else {
            continue;
        };
        if meta.starts_with("160000 ") && fs::try_exists(path.join(name).join(".git")).await? {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn loose(path: &Path) -> Result<bool, Error> {
    if git(path, ["symbolic-ref", "--quiet", "HEAD"]).await.is_ok() {
        return Ok(false);
    }

    let containing = git(
        path,
        [
            "for-each-ref",
            "--count=1",
            "--contains=HEAD",
            "--format=%(refname)",
            "refs/heads",
            "refs/remotes",
        ],
    )
    .await?;

    Ok(containing.is_empty())
}

async fn linked(repo: &Path, path: &Path) -> Result<bool, Error> {
    let wanted = fs::canonicalize(path).await.ok();
    for worktree in worktrees(repo).await? {
        if worktree.main {
            continue;
        }

        let real = fs::canonicalize(&worktree.path).await.ok();
        let same = match (&wanted, &real) {
            (Some(wanted), Some(real)) => wanted == real,
            _ => worktree.path == path,
        };
        if same {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn release(path: &Path) -> Result<(), Error> {
    let mut tries = 0;
    loop {
        match fs::remove_dir_all(path).await {
            Ok(()) => {
                return Ok(());
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) if tries == RELEASE_TRIES => {
                return Err(Error::Git(format!(
                    "removed the worktree, but could not delete {}: {error}",
                    path.display()
                )));
            }
            Err(_) => {
                tries += 1;
                tokio::time::sleep(RELEASE_WAIT).await;
            }
        }
    }
}

async fn undo(repo: &Path, path: &Path, branch: Option<&str>) {
    let removed = git(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("remove"),
            OsStr::new("--force"),
            path.as_os_str(),
        ],
    )
    .await;
    if let Err(error) = removed {
        tracing::warn!(%error, "could not remove a worktree that failed to set up");
        return;
    }

    if let Some(branch) = branch
        && let Err(error) = git(repo, ["branch", "--delete", "--force", branch]).await
    {
        tracing::warn!(%error, "could not delete the branch of a worktree that failed to set up");
    }
}

#[cfg(test)]
mod tests;
