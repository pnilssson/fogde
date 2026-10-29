use std::path::Path;
use std::time::SystemTime;

use tokio::fs;

use crate::Error;
use crate::remote::{config, remotes};
use crate::run::{git, git_paths};
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
    let path = git_paths(root, &["FETCH_HEAD"]).await.ok()?.pop()?;

    fs::metadata(path).await.ok()?.modified().ok()
}

pub async fn switch(root: &Path, branch: &str) -> Result<(), Error> {
    if branch.is_empty() || branch.starts_with('-') {
        return Err(Error::Git(format!("{branch} is not a branch name")));
    }

    git(root, ["switch", "--quiet", branch]).await.map(drop)
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
