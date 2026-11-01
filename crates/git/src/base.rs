use std::cmp::Reverse;
use std::path::Path;
use std::time::Duration;

use crate::Error;
use crate::remote::{Remote, config};
use crate::run::{git, git_tree};

const LIMIT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mainline {
    pub(crate) name: String,
    pub(crate) reference: String,
    pub(crate) remote: String,
}

pub(crate) fn remote(remotes: &[Remote]) -> Option<&Remote> {
    let mut ranked: Vec<&Remote> = remotes.iter().collect();
    ranked.sort_by_key(|remote| Reverse(rank(&remote.name)));
    if let Some(resolved) = ranked
        .iter()
        .copied()
        .find(|remote| remote.resolved.is_some())
    {
        return Some(resolved);
    }

    let single = ranked.len() == 1;

    ranked
        .into_iter()
        .next()
        .filter(|remote| single || rank(&remote.name) > 0)
}

pub(crate) async fn mainline(repo: &Path) -> Option<Mainline> {
    let remotes = config(repo, None).await.remotes;
    let first = remote(&remotes)?;
    let mut others: Vec<&Remote> = remotes
        .iter()
        .filter(|remote| remote.name != first.name && rank(&remote.name) > 0)
        .collect();
    others.sort_by_key(|remote| Reverse(rank(&remote.name)));

    for remote in [first].into_iter().chain(others) {
        if let Some(found) = resolve(repo, remote).await {
            return Some(found);
        }
    }

    None
}

async fn resolve(repo: &Path, remote: &Remote) -> Option<Mainline> {
    let prefix = format!("refs/remotes/{}/", remote.name);

    let head = format!("{prefix}HEAD");
    if let Ok(target) = read(repo, &["symbolic-ref", "--quiet", &head]).await
        && let Some(name) = target.strip_prefix(&prefix)
    {
        return Some(Mainline {
            name: name.to_owned(),
            reference: target.clone(),
            remote: remote.name.clone(),
        });
    }

    for name in ["main", "master"] {
        let reference = format!("{prefix}{name}");
        if read(repo, &["rev-parse", "--verify", "--quiet", &reference])
            .await
            .is_ok()
        {
            return Some(Mainline {
                name: name.to_owned(),
                reference,
                remote: remote.name.clone(),
            });
        }
    }

    None
}

pub async fn fetch(repo: &Path) -> Result<(), Error> {
    fetch_within(repo, LIMIT).await
}

async fn fetch_within(repo: &Path, limit: Duration) -> Result<(), Error> {
    let branch = read(repo, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .await
        .ok();
    let config = config(repo, branch.as_deref()).await;
    let mut names: Vec<&str> = Vec::new();
    if let Some(base) = remote(&config.remotes) {
        names.push(&base.name);
    }
    if let Some(tracking) = config.tracking.as_deref()
        && config.remotes.iter().any(|remote| remote.name == tracking)
        && !names.contains(&tracking)
    {
        names.push(tracking);
    }
    let resolved = mainline(repo).await.map(|mainline| mainline.remote);
    if let Some(resolved) = resolved.as_deref()
        && !names.contains(&resolved)
    {
        names.push(resolved);
    }
    if names.is_empty() {
        return Err(Error::Git("add a remote before fetching".to_owned()));
    }

    let args = ["fetch", "--quiet", "--multiple"].into_iter().chain(names);
    match tokio::time::timeout(limit, git_tree(repo, args)).await {
        Ok(fetched) => fetched,
        Err(_) => Err(Error::Git("fetching took too long".to_owned())),
    }
}

fn rank(name: &str) -> u8 {
    match name.to_ascii_lowercase().as_str() {
        "upstream" => 3,
        "github" => 2,
        "origin" => 1,
        _ => 0,
    }
}

async fn read(repo: &Path, args: &[&str]) -> Result<String, Error> {
    let output = git(repo, ["--no-optional-locks"].iter().chain(args)).await?;

    Ok(String::from_utf8_lossy(&output).trim().to_owned())
}

#[cfg(test)]
mod tests;
