use std::path::Path;

use crate::Error;
use crate::run::git;

const BRANCHES: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub worktree: Option<String>,
    pub remote: bool,
    pub date: String,
}

pub async fn branches(root: &Path) -> Result<Vec<Branch>, Error> {
    let output = git(
        root,
        [
            "for-each-ref",
            "--sort=-committerdate",
            "--format=%(refname)%00%(worktreepath)%00%(committerdate:iso-strict)",
            "refs/heads",
            "refs/remotes",
        ],
    )
    .await?;

    Ok(parse(&String::from_utf8_lossy(&output)))
}

fn parse(output: &str) -> Vec<Branch> {
    let mut branches: Vec<Branch> = Vec::new();
    for line in output.lines() {
        let mut fields = line.split('\0');
        let (Some(reference), Some(worktree), Some(date)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let (name, remote) = if let Some(local) = reference.strip_prefix("refs/heads/") {
            (local, false)
        } else if let Some(remote) = reference.strip_prefix("refs/remotes/") {
            match remote.split_once('/') {
                Some((_, name)) if name != "HEAD" => (name, true),
                _ => continue,
            }
        } else {
            continue;
        };

        let worktree = (!worktree.is_empty()).then(|| worktree.to_owned());
        if let Some(known) = branches.iter_mut().find(|known| known.name == name) {
            if !remote {
                known.remote = false;
                known.worktree = worktree;
            }
            continue;
        }
        if branches.len() == BRANCHES {
            continue;
        }

        branches.push(Branch {
            name: name.to_owned(),
            worktree,
            remote,
            date: date.to_owned(),
        });
    }

    branches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_each_branch_once_with_its_worktree() {
        let output = "refs/remotes/origin/feature/login\x00\x002026-10-29T10:00:00+02:00\n\
            refs/heads/main\x00/repos/app\x002026-10-28T10:00:00+02:00\n\
            refs/heads/feature/login\x00/worktrees/app/login\x002026-10-27T10:00:00+02:00\n\
            refs/remotes/origin/HEAD\x00\x002026-10-28T10:00:00+02:00\n\
            refs/remotes/origin/fix\x00\x002026-10-26T10:00:00+02:00\n\
            refs/tags/v1\x00\x002026-10-25T10:00:00+02:00\n";

        assert_eq!(
            parse(output),
            [
                Branch {
                    name: "feature/login".to_owned(),
                    worktree: Some("/worktrees/app/login".to_owned()),
                    remote: false,
                    date: "2026-10-29T10:00:00+02:00".to_owned(),
                },
                Branch {
                    name: "main".to_owned(),
                    worktree: Some("/repos/app".to_owned()),
                    remote: false,
                    date: "2026-10-28T10:00:00+02:00".to_owned(),
                },
                Branch {
                    name: "fix".to_owned(),
                    worktree: None,
                    remote: true,
                    date: "2026-10-26T10:00:00+02:00".to_owned(),
                },
            ]
        );
    }
}
