use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tokio::fs;

use crate::Error;

const SHARED: [&str; 10] = [
    "config",
    "hooks",
    "info",
    "logs",
    "objects",
    "packed-refs",
    "refs",
    "remotes",
    "shallow",
    "worktrees",
];
const OWN: [&str; 1] = ["logs/HEAD"];
const STAMPED: [&str; 6] = [
    "HEAD",
    "index",
    "logs/HEAD",
    "FETCH_HEAD",
    "packed-refs",
    "reftable/tables.list",
];

pub(crate) struct Dirs {
    pub(crate) root: PathBuf,
    pub(crate) git: PathBuf,
    pub(crate) common: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp(Vec<Option<SystemTime>>);

impl Dirs {
    pub(crate) async fn find(folder: &Path) -> Option<Self> {
        for dir in folder.ancestors() {
            if let Some(git) = git_dir(dir).await {
                let common = match fs::read_to_string(git.join("commondir")).await {
                    Ok(link) => git.join(link.trim()),
                    Err(_) => git.clone(),
                };

                return Some(Self {
                    root: dir.to_owned(),
                    git,
                    common,
                });
            }
        }

        None
    }

    pub(crate) async fn of(folder: &Path) -> Result<Self, Error> {
        Self::find(folder)
            .await
            .ok_or_else(|| Error::Git(format!("{} is not a Git repository", folder.display())))
    }

    pub(crate) fn path(&self, name: &str) -> PathBuf {
        let first = name.split('/').next().unwrap_or(name);
        let base = if SHARED.contains(&first) && !OWN.contains(&name) {
            &self.common
        } else {
            &self.git
        };

        name.split('/')
            .fold(base.clone(), |path, part| path.join(part))
    }
}

pub async fn stamp(folder: &Path) -> Option<Stamp> {
    let dirs = Dirs::find(folder).await?;
    let mut times = Vec::with_capacity(STAMPED.len());
    for name in STAMPED {
        let modified = fs::metadata(dirs.path(name))
            .await
            .and_then(|metadata| metadata.modified());
        times.push(modified.ok());
    }

    Some(Stamp(times))
}

async fn git_dir(dir: &Path) -> Option<PathBuf> {
    let dot = dir.join(".git");
    let metadata = fs::metadata(&dot).await.ok()?;
    if metadata.is_dir() {
        return Some(dot);
    }

    let link = fs::read_to_string(&dot).await.ok()?;
    let target = link.trim().strip_prefix("gitdir:")?.trim();

    Some(dir.join(target))
}

#[cfg(test)]
mod tests;
