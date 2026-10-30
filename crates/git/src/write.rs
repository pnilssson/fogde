use std::path::Path;
use std::time::SystemTime;

use tokio::fs;

use crate::Error;
use crate::head::common;
use crate::remote::{config, remotes};
use crate::run::git;
use crate::status::status;

pub async fn push(root: &Path) -> Result<(), Error> {
    let current = status(root).await?;
    let Some(branch) = current.branch else {
        return Err(Error::Git("switch to a branch before pushing".to_owned()));
    };
    if current.upstream.is_some() {
        return git(root, ["push", "--quiet"]).await.map(drop);
    }

    let remote = match config(root, Some(&branch)).await.push {
        Some(remote) => remote,
        None => default_remote(root).await?,
    };

    git(root, ["push", "--quiet", "--set-upstream", &remote, "HEAD"])
        .await
        .map(drop)
}

pub async fn pull(root: &Path) -> Result<(), Error> {
    let current = status(root).await?;
    if current.upstream.is_none() {
        return Err(Error::Git("push the branch before pulling".to_owned()));
    }

    git(root, ["fetch", "--quiet"]).await?;
    let fetched = status(root).await?;
    if fetched.ahead > 0 && fetched.behind > 0 {
        return Err(Error::Git(
            "the branch has diverged from its upstream; merge or rebase it in a terminal"
                .to_owned(),
        ));
    }

    git(root, ["merge", "--ff-only", "--quiet", "@{upstream}"])
        .await
        .map(drop)
}

pub async fn refresh(root: &Path) -> Result<(), Error> {
    if remotes(root).await.is_empty() {
        return Err(Error::Git("add a remote before fetching".to_owned()));
    }

    git(root, ["fetch", "--quiet"]).await.map(drop)
}

pub async fn fetched(root: &Path) -> Option<SystemTime> {
    let common = common(root).await?;
    let mut heads = vec![common.join("FETCH_HEAD")];
    if let Ok(mut entries) = fs::read_dir(common.join("worktrees")).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            heads.push(entry.path().join("FETCH_HEAD"));
        }
    }

    let mut newest = None;
    for head in heads {
        let Ok(modified) = fs::metadata(head).await.and_then(|meta| meta.modified()) else {
            continue;
        };
        newest = newest.max(Some(modified));
    }

    newest
}

pub async fn switch(root: &Path, branch: &str) -> Result<(), Error> {
    if branch.is_empty() || branch.starts_with('-') {
        return Err(Error::Git(format!("{branch} is not a branch name")));
    }

    git(root, ["switch", "--quiet", branch]).await.map(drop)
}

pub async fn branch_off(root: &Path, name: &str) -> Result<(), Error> {
    let name = name.trim();
    let valid = !name.is_empty()
        && !name.starts_with('-')
        && git(root, ["check-ref-format", "--branch", name])
            .await
            .is_ok();
    if !valid {
        return Err(Error::Git(format!("{name} is not a valid branch name")));
    }

    git(root, ["switch", "--quiet", "--create", name])
        .await
        .map(drop)
}

async fn default_remote(root: &Path) -> Result<String, Error> {
    let remotes = remotes(root).await;
    if let Some(origin) = remotes.iter().find(|remote| remote.name == "origin") {
        return Ok(origin.name.clone());
    }

    match remotes.as_slice() {
        [only] => Ok(only.name.clone()),
        [] => Err(Error::Git("add a remote before pushing".to_owned())),
        _ => Err(Error::Git(
            "choose a remote with git push --set-upstream in a terminal".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests;
