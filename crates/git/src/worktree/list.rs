use std::path::PathBuf;

use super::Worktree;

pub(super) fn parse(porcelain: &str) -> Vec<Worktree> {
    let mut found: Vec<(Worktree, bool)> = Vec::new();

    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            found.push((
                Worktree {
                    path: native(path),
                    branch: None,
                    main: found.is_empty(),
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
pub(super) fn native(path: &str) -> PathBuf {
    PathBuf::from(path.replace('/', "\\"))
}

#[cfg(not(windows))]
pub(super) fn native(path: &str) -> PathBuf {
    PathBuf::from(path)
}
