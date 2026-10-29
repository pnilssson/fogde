use std::path::Path;

use crate::Error;
use crate::run::{failure, git, git_with_input};
use crate::status::{Change, status};

pub async fn commit(root: &Path, paths: &[String], message: &str) -> Result<(), Error> {
    if message.trim().is_empty() {
        return Err(Error::Git("write a commit message".to_owned()));
    }
    if paths.is_empty() {
        return Err(Error::Git("choose the files to commit".to_owned()));
    }

    let current = status(root).await?;
    let chosen = |path: &String| paths.contains(path);
    let conflicted = current
        .files
        .iter()
        .any(|file| file.change == Change::Conflicted && chosen(&file.path));
    if conflicted || merging(root).await {
        return Err(Error::Git(
            "finish or abort the merge in a terminal first".to_owned(),
        ));
    }

    let untracked: Vec<&str> = current
        .files
        .iter()
        .filter(|file| file.change == Change::Untracked && chosen(&file.path))
        .map(|file| file.path.as_str())
        .collect();
    if !untracked.is_empty() {
        let mut add = vec!["--literal-pathspecs", "add", "--"];
        add.extend(&untracked);
        git(root, add).await?;
    }

    let result = record(root, paths, message).await;
    if result.is_err() && !untracked.is_empty() {
        let mut unstage = vec![
            "--literal-pathspecs",
            "rm",
            "-r",
            "--cached",
            "--force",
            "--quiet",
            "--ignore-unmatch",
            "--",
        ];
        unstage.extend(&untracked);
        if let Err(error) = git(root, unstage).await {
            tracing::warn!(%error, "could not unstage the new files of a failed commit");
        }
    }

    result
}

async fn record(root: &Path, paths: &[String], message: &str) -> Result<(), Error> {
    let mut args = vec![
        "--literal-pathspecs",
        "commit",
        "--quiet",
        "--only",
        "--file",
        "-",
        "--",
    ];
    args.extend(paths.iter().map(String::as_str));
    let output = git_with_input(root, args, message.as_bytes()).await?;
    if !output.status.success() {
        return Err(failure(&output));
    }

    Ok(())
}

async fn merging(root: &Path) -> bool {
    git(root, ["rev-parse", "--quiet", "--verify", "MERGE_HEAD"])
        .await
        .is_ok()
}

#[cfg(test)]
mod tests;
