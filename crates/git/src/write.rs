use std::path::Path;
use std::time::SystemTime;

use tokio::fs;

use crate::Error;
use crate::head::admins;
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

pub async fn fetched(root: &Path) -> Option<SystemTime> {
    let mut newest = None;
    for (_, dir) in admins(root).await {
        let head = dir.join("FETCH_HEAD");
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
