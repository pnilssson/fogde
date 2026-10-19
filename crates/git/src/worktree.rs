use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::include;
use crate::run::git;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
}

pub async fn worktrees(repo: &Path) -> Result<Vec<Worktree>, Error> {
    let listed = git(repo, ["worktree", "list", "--porcelain"]).await?;

    Ok(parse(&String::from_utf8_lossy(&listed)))
}

pub async fn add_worktree(repo: &Path, path: &Path, branch: &str) -> Result<(), Error> {
    let reference = format!("refs/heads/{branch}");
    let existing = git(repo, ["show-ref", "--verify", "--quiet", &reference])
        .await
        .is_ok();
    if existing {
        git(
            repo,
            [
                OsStr::new("worktree"),
                OsStr::new("add"),
                path.as_os_str(),
                OsStr::new(branch),
            ],
        )
        .await?;
    } else {
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
    }

    let Err(error) = include::copy(repo, path).await else {
        return Ok(());
    };
    undo(repo, path, (!existing).then_some(branch)).await;

    Err(error)
}

pub async fn remove_worktree(repo: &Path, path: &Path) -> Result<Option<String>, Error> {
    let branch = git(path, ["symbolic-ref", "--quiet", "--short", "HEAD"])
        .await
        .ok()
        .map(|branch| String::from_utf8_lossy(&branch).trim().to_owned());

    git(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("remove"),
            path.as_os_str(),
        ],
    )
    .await?;

    Ok(branch)
}

pub async fn remove_merged_branch(repo: &Path, branch: &str) -> Result<bool, Error> {
    let tip = format!("refs/heads/{branch}");
    let mut merged = false;
    for base in ["HEAD", "refs/remotes/origin/HEAD"] {
        if git(repo, ["merge-base", "--is-ancestor", &tip, base])
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
    if let Err(error) = git(repo, ["fetch", "--quiet", "origin"]).await {
        tracing::warn!(%error, "could not fetch before adding a worktree");
    }

    git(
        repo,
        [
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )
    .await
    .map_or_else(
        |_| "HEAD".to_owned(),
        |base| String::from_utf8_lossy(&base).trim().to_owned(),
    )
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

fn parse(porcelain: &str) -> Vec<Worktree> {
    let mut found: Vec<(Worktree, bool)> = Vec::new();

    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            found.push((
                Worktree {
                    path: native(path),
                    branch: None,
                },
                true,
            ));
        } else if let Some((worktree, usable)) = found.last_mut() {
            if let Some(branch) = line.strip_prefix("branch ") {
                worktree.branch = Some(branch.trim_start_matches("refs/heads/").to_owned());
            } else if line == "bare" || line.starts_with("prunable") {
                *usable = false;
            }
        }
    }

    found
        .into_iter()
        .filter_map(|(worktree, usable)| usable.then_some(worktree))
        .collect()
}

#[cfg(windows)]
fn native(path: &str) -> PathBuf {
    PathBuf::from(path.replace('/', "\\"))
}

#[cfg(not(windows))]
fn native(path: &str) -> PathBuf {
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_worktrees_and_skips_unusable_ones() {
        let porcelain = "worktree /repos/fogde\nHEAD 3f2a\nbranch refs/heads/main\n\n\
            worktree /Fogde/worktrees/fogde/login\nHEAD 9b1c\nbranch refs/heads/fogde/login\n\n\
            worktree /Fogde/worktrees/fogde/gone\nHEAD 77aa\nbranch refs/heads/fogde/gone\n\
            prunable gitdir file points to non-existent location\n\n\
            worktree /tmp/review\nHEAD 1d0e\ndetached\n\n";

        assert_eq!(
            parse(porcelain),
            [
                Worktree {
                    path: native("/repos/fogde"),
                    branch: Some("main".to_owned()),
                },
                Worktree {
                    path: native("/Fogde/worktrees/fogde/login"),
                    branch: Some("fogde/login".to_owned()),
                },
                Worktree {
                    path: native("/tmp/review"),
                    branch: None,
                },
            ]
        );
    }

    #[test]
    fn reads_windows_line_endings() {
        let porcelain = "worktree C:/Users/dev/fogde\r\nHEAD 3f2a\r\nbranch refs/heads/main\r\n\r\n";

        assert_eq!(
            parse(porcelain),
            [Worktree {
                path: native("C:/Users/dev/fogde"),
                branch: Some("main".to_owned()),
            }]
        );
    }
}
