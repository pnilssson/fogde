use std::path::Path;

use tokio::fs;

use crate::Error;
use crate::run::git;

const UNTRACKED_FILES: usize = 500;
const UNTRACKED_SIZE: u64 = 256 * 1024;
const UNTRACKED_BUDGET: u64 = 2 * 1024 * 1024;
const BINARY_PROBE: usize = 8000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changes {
    pub added: u32,
    pub removed: u32,
    pub base: Option<Base>,
    pub upstream: Option<Divergence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base {
    pub name: String,
    pub behind: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub ahead: u32,
    pub behind: u32,
}

pub async fn changes(root: &Path) -> Option<Changes> {
    let remote = remote_head(root).await;
    let merged = match &remote {
        Some(remote) => read(root, ["merge-base", "HEAD", remote]).await.ok(),
        None => None,
    };
    let base = remote.filter(|_| merged.is_some());
    let start = match merged {
        Some(merged) => merged,
        None => read(root, ["rev-parse", "--verify", "--quiet", "HEAD"])
            .await
            .ok()?,
    };
    let (diffed, untracked, upstream, behind) = tokio::join!(
        read(root, ["diff", "--numstat", &start]),
        untracked(root),
        divergence(root, "@{upstream}"),
        async {
            match &base {
                Some(base) => divergence(root, base).await,
                None => None,
            }
        },
    );
    let (added, removed) = numstat(&diffed.ok()?);

    Some(Changes {
        added: added.saturating_add(untracked),
        removed,
        base: base.zip(behind).map(|(name, divergence)| Base {
            name: name.strip_prefix("origin/").unwrap_or(&name).to_owned(),
            behind: divergence.behind,
        }),
        upstream,
    })
}

pub async fn fetch(repo: &Path) -> Result<(), Error> {
    git(repo, ["fetch", "--quiet", "origin"]).await.map(drop)
}

pub(crate) async fn remote_head(repo: &Path) -> Option<String> {
    let head = read(
        repo,
        [
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )
    .await;
    if let Ok(head) = head {
        return Some(head);
    }

    for name in ["origin/main", "origin/master"] {
        let reference = format!("refs/remotes/{name}");
        if read(repo, ["rev-parse", "--verify", "--quiet", &reference])
            .await
            .is_ok()
        {
            return Some(name.to_owned());
        }
    }

    None
}

async fn divergence(root: &Path, other: &str) -> Option<Divergence> {
    let range = format!("HEAD...{other}");
    let counted = read(root, ["rev-list", "--left-right", "--count", &range])
        .await
        .ok()?;
    let (ahead, behind) = counted.split_once(char::is_whitespace)?;

    Some(Divergence {
        ahead: ahead.trim().parse().ok()?,
        behind: behind.trim().parse().ok()?,
    })
}

async fn untracked(root: &Path) -> u32 {
    let Ok(listed) = git(
        root,
        [
            "--no-optional-locks",
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )
    .await
    else {
        return 0;
    };

    let mut total: u32 = 0;
    let mut budget = UNTRACKED_BUDGET;
    for path in String::from_utf8_lossy(&listed)
        .split('\0')
        .filter(|path| !path.is_empty())
        .take(UNTRACKED_FILES)
    {
        let path = root.join(path);
        let Ok(metadata) = fs::metadata(&path).await else {
            continue;
        };
        if !metadata.is_file() || metadata.len() > UNTRACKED_SIZE {
            continue;
        }
        if metadata.len() > budget {
            break;
        }

        budget -= metadata.len();
        if let Some(lines) = fs::read(&path)
            .await
            .ok()
            .and_then(|bytes| count_lines(&bytes))
        {
            total = total.saturating_add(lines);
        }
    }

    total
}

fn count_lines(bytes: &[u8]) -> Option<u32> {
    if bytes[..bytes.len().min(BINARY_PROBE)].contains(&0) {
        return None;
    }

    let newlines = bytes.iter().fold(0u32, |count, &byte| {
        count.saturating_add(u32::from(byte == b'\n'))
    });
    let unterminated = bytes.last().is_some_and(|&byte| byte != b'\n');

    Some(newlines.saturating_add(u32::from(unterminated)))
}

fn numstat(output: &str) -> (u32, u32) {
    output.lines().fold((0, 0), |(added, removed), line| {
        let mut fields = line
            .split('\t')
            .map(|field| field.parse::<u32>().unwrap_or(0));
        let more = fields.next().unwrap_or(0);
        let fewer = fields.next().unwrap_or(0);

        (added.saturating_add(more), removed.saturating_add(fewer))
    })
}

async fn read<const N: usize>(repo: &Path, args: [&str; N]) -> Result<String, Error> {
    let output = git(repo, ["--no-optional-locks"].into_iter().chain(args)).await?;

    Ok(String::from_utf8_lossy(&output).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_text_lines_and_skips_binary_files() {
        let output = "12\t3\tsrc/main.rs\n-\t-\tlogo.png\n0\t7\tsrc/{old.rs => new.rs}\n";

        assert_eq!(numstat(output), (12, 10));
        assert_eq!(numstat(""), (0, 0));
    }

    #[test]
    fn counts_lines_with_or_without_a_final_newline() {
        assert_eq!(count_lines(b"one\ntwo\n"), Some(2));
        assert_eq!(count_lines(b"one\ntwo"), Some(2));
        assert_eq!(count_lines(b""), Some(0));
        assert_eq!(count_lines(b"\x89PNG\0\0"), None);
    }
}
