use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::fs;

use crate::Error;
use crate::diff::{Diff, between};
use crate::dirs::Dirs;
use crate::run::{git, git_env};

static SCRATCH: AtomicU64 = AtomicU64::new(0);

pub async fn snapshot(root: &Path, store: &Path) -> Result<String, Error> {
    let repo = Repo::find(root).await?;
    let objects = store.join("objects");
    fs::create_dir_all(&objects).await?;
    let alternates = env::join_paths([&repo.objects]).map_err(|error| unjoinable(&error))?;

    capture(root, store, &repo, &objects, &alternates).await
}

pub async fn turn(root: &Path, store: &Path, start: &str) -> Result<Diff, Error> {
    let repo = Repo::find(root).await?;
    let scratch = scratch(store);
    fs::create_dir_all(&scratch).await?;

    let compared = compare(root, store, start, &repo, &scratch).await;
    if let Err(error) = fs::remove_dir_all(&scratch).await {
        tracing::debug!(%error, "could not remove the objects of a turn diff");
    }

    compared
}

async fn compare(
    root: &Path,
    store: &Path,
    start: &str,
    repo: &Repo,
    scratch: &Path,
) -> Result<Diff, Error> {
    let alternates = env::join_paths([&store.join("objects"), &repo.objects])
        .map_err(|error| unjoinable(&error))?;
    let now = capture(root, store, repo, scratch, &alternates).await?;
    let env = [
        ("GIT_OBJECT_DIRECTORY", scratch.as_os_str()),
        ("GIT_ALTERNATE_OBJECT_DIRECTORIES", alternates.as_os_str()),
    ];

    between(root, start, &now, &env).await
}

async fn capture(
    root: &Path,
    store: &Path,
    repo: &Repo,
    objects: &Path,
    alternates: &OsStr,
) -> Result<String, Error> {
    let index = scratch(store);
    if fs::try_exists(&repo.index).await.unwrap_or(false) {
        fs::copy(&repo.index, &index).await?;
    }

    let env = [
        ("GIT_OBJECT_DIRECTORY", objects.as_os_str()),
        ("GIT_ALTERNATE_OBJECT_DIRECTORIES", alternates),
        ("GIT_INDEX_FILE", index.as_os_str()),
    ];
    let mut safe = vec![
        "-c".to_owned(),
        "core.fsmonitor=false".to_owned(),
        "-c".to_owned(),
        "core.splitIndex=false".to_owned(),
        "-c".to_owned(),
        format!("core.hooksPath={}", store.join("no-hooks").display()),
    ];
    safe.extend(unfiltered(root).await);
    let add = [safe.as_slice(), &["add".to_owned(), "--all".to_owned()]].concat();
    let write = [safe.as_slice(), &["write-tree".to_owned()]].concat();
    let tree = async {
        git_env(root, &add, &env).await?;
        git_env(root, &write, &env).await
    }
    .await;
    let _ = fs::remove_file(&index).await;

    Ok(String::from_utf8_lossy(&tree?).trim().to_owned())
}

async fn unfiltered(root: &Path) -> Vec<String> {
    let Ok(output) = git(
        root,
        [
            "config",
            "--name-only",
            "--get-regexp",
            r"^filter\..*\.(clean|process|required)$",
        ],
    )
    .await
    else {
        return Vec::new();
    };

    let mut drivers: Vec<String> = String::from_utf8_lossy(&output)
        .lines()
        .filter_map(|key| {
            let (driver, _) = key.strip_prefix("filter.")?.rsplit_once('.')?;

            Some(driver.to_owned())
        })
        .collect();
    drivers.sort();
    drivers.dedup();

    drivers
        .iter()
        .flat_map(|driver| {
            ["clean=", "process=", "required=false"]
                .map(|setting| ["-c".to_owned(), format!("filter.{driver}.{setting}")])
        })
        .flatten()
        .collect()
}

fn scratch(dir: &Path) -> PathBuf {
    dir.join(format!(
        "scratch-{}-{}",
        std::process::id(),
        SCRATCH.fetch_add(1, Ordering::Relaxed)
    ))
}

fn unjoinable(error: &env::JoinPathsError) -> Error {
    Error::Git(format!("could not use the repository's objects: {error}"))
}

struct Repo {
    objects: PathBuf,
    index: PathBuf,
}

impl Repo {
    async fn find(root: &Path) -> Result<Self, Error> {
        let dirs = Dirs::of(root).await?;

        Ok(Self {
            objects: dirs.path("objects"),
            index: dirs.path("index"),
        })
    }
}

#[cfg(test)]
mod tests;
