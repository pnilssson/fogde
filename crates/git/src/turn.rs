use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::fs;

use crate::Error;
use crate::diff::{Diff, between};
use crate::run::{git_env, git_paths};

static SCRATCH: AtomicU64 = AtomicU64::new(0);

pub async fn snapshot(root: &Path, store: &Path) -> Result<String, Error> {
    let repo = Repo::find(root, store).await?;

    capture(root, store, &repo).await
}

pub async fn turn(root: &Path, store: &Path, start: &str) -> Result<Diff, Error> {
    let repo = Repo::find(root, store).await?;
    let now = capture(root, store, &repo).await?;

    between(root, start, &now, &repo.env()).await
}

async fn capture(root: &Path, store: &Path, repo: &Repo) -> Result<String, Error> {
    let scratch = store.join(format!(
        "index-{}-{}",
        std::process::id(),
        SCRATCH.fetch_add(1, Ordering::Relaxed)
    ));
    if fs::try_exists(&repo.index).await.unwrap_or(false) {
        fs::copy(&repo.index, &scratch).await?;
    }

    let tree = write_tree(root, repo, &scratch).await;
    let _ = fs::remove_file(&scratch).await;

    tree
}

struct Repo {
    objects: PathBuf,
    index: PathBuf,
    own: PathBuf,
}

impl Repo {
    async fn find(root: &Path, store: &Path) -> Result<Self, Error> {
        let paths = git_paths(root, &["objects", "index"]).await?;
        let [objects, index] = paths.as_slice() else {
            return Err(Error::Git("could not find the repository".to_owned()));
        };
        let own = store.join("objects");
        fs::create_dir_all(&own).await?;

        Ok(Self {
            objects: objects.clone(),
            index: index.clone(),
            own,
        })
    }

    fn env(&self) -> [(&'static str, &OsStr); 2] {
        [
            ("GIT_OBJECT_DIRECTORY", self.own.as_os_str()),
            ("GIT_ALTERNATE_OBJECT_DIRECTORIES", self.objects.as_os_str()),
        ]
    }
}

async fn write_tree(root: &Path, repo: &Repo, scratch: &Path) -> Result<String, Error> {
    let [objects, alternates] = repo.env();
    let env = [objects, alternates, ("GIT_INDEX_FILE", scratch.as_os_str())];
    git_env(root, ["-c", "core.fsmonitor=false", "add", "--all"], &env).await?;
    let tree = git_env(root, ["write-tree"], &env).await?;

    Ok(String::from_utf8_lossy(&tree).trim().to_owned())
}

#[cfg(test)]
mod tests;
