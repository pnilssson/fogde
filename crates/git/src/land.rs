use std::path::Path;

use crate::Error;
use crate::run::git;
use crate::status::status;
use crate::worktree::clean;
use crate::write::{pull, switch};

pub async fn landed(root: &Path, sha: &str, base: &str) -> Result<String, Error> {
    if !clean(root).await? {
        return Err(Error::Git("commit or discard the changes first".to_owned()));
    }

    let Some(branch) = status(root).await?.branch else {
        return Err(Error::Git(
            "switch to the pull request's branch first".to_owned(),
        ));
    };
    if branch == base {
        return Err(Error::Git(format!("already on {base}")));
    }
    let head = git(root, ["rev-parse", "HEAD"]).await?;
    if String::from_utf8_lossy(&head).trim() != sha {
        return Err(Error::Git(format!(
            "{branch} has commits that are not in the pull request"
        )));
    }

    Ok(branch)
}

pub async fn leave(root: &Path, branch: &str, base: &str) -> Result<(), Error> {
    switch(root, base).await?;
    delete_branch(root, branch).await?;
    if status(root).await?.upstream.is_none() {
        return Ok(());
    }

    pull(root)
        .await
        .map_err(|error| Error::Git(format!("switched to {base}, but could not pull: {error}")))
}

pub async fn delete_branch(repo: &Path, branch: &str) -> Result<(), Error> {
    if branch.starts_with('-') {
        return Err(Error::Git(format!("{branch} is not a branch name")));
    }

    git(repo, ["branch", "--delete", "--force", branch])
        .await
        .map(drop)
}

#[cfg(test)]
mod tests;
