mod parse;

use std::path::Path;

use tokio::fs;

use crate::Error;
use crate::changes::{binary, remote_head};
use crate::run::{git, git_with_input};
use crate::status::{Change, status};

const FILE_LIMIT: usize = 512 * 1024;
const LINE_LIMIT: u32 = 20_000;
const TOTAL_LIMIT: usize = 8 * 1024 * 1024;
const UNTRACKED_FILES: usize = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Uncommitted,
    Branch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff {
    pub base: Option<String>,
    pub files: Vec<FileDiff>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    pub path: String,
    pub from: Option<String>,
    pub change: Change,
    pub added: u32,
    pub removed: u32,
    pub patch: Option<String>,
    pub binary: bool,
    pub large: bool,
}

pub async fn diff(root: &Path, scope: Scope) -> Result<Diff, Error> {
    let (start, base) = start(root, scope).await?;
    let output = git(
        root,
        [
            "-c",
            "core.quotePath=false",
            "-c",
            "diff.noprefix=false",
            "-c",
            "diff.mnemonicPrefix=false",
            "--no-optional-locks",
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-relative",
            "--find-renames",
            "--unified=3",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            &start,
            "--",
        ],
    )
    .await?;

    let mut chunks: Vec<(parse::Chunk, bool)> = parse::split(&String::from_utf8_lossy(&output))
        .into_iter()
        .map(|chunk| (chunk, false))
        .collect();
    let (added, left_out) = untracked(root).await?;
    chunks.extend(added);
    chunks.sort_by(|(a, _), (b, _)| a.path.cmp(&b.path));

    let mut diff = limit(base, chunks);
    diff.truncated |= left_out;

    Ok(diff)
}

async fn start(root: &Path, scope: Scope) -> Result<(String, Option<String>), Error> {
    if scope == Scope::Branch
        && let Some(remote) = remote_head(root).await
        && let Ok(merged) = git(root, ["merge-base", "HEAD", &remote]).await
    {
        let name = remote
            .split_once('/')
            .map_or(remote.as_str(), |(_, name)| name);

        return Ok((
            String::from_utf8_lossy(&merged).trim().to_owned(),
            Some(name.to_owned()),
        ));
    }

    if git(root, ["rev-parse", "--verify", "--quiet", "HEAD"])
        .await
        .is_ok()
    {
        return Ok(("HEAD".to_owned(), None));
    }

    let empty = git_with_input(root, ["hash-object", "-t", "tree", "--stdin"], &[]).await?;

    Ok((
        String::from_utf8_lossy(&empty.stdout).trim().to_owned(),
        None,
    ))
}

async fn untracked(root: &Path) -> Result<(Vec<(parse::Chunk, bool)>, bool), Error> {
    let files = status(root).await?.files;
    let mut chunks = Vec::new();
    let mut left = TOTAL_LIMIT;
    for file in files
        .into_iter()
        .filter(|file| file.change == Change::Untracked)
    {
        if chunks.len() == UNTRACKED_FILES {
            return Ok((chunks, true));
        }

        let full = root.join(&file.path);
        let Ok(metadata) = fs::symlink_metadata(&full).await else {
            continue;
        };
        let linked = metadata.is_symlink();
        if !linked && !metadata.is_file() {
            continue;
        }

        let large =
            !linked && usize::try_from(metadata.len()).map_or(true, |size| size > FILE_LIMIT);
        let bytes = if linked {
            fs::read_link(&full)
                .await
                .map(|target| target.into_os_string().into_encoded_bytes())
                .unwrap_or_default()
        } else if large {
            Vec::new()
        } else {
            fs::read(&full).await.unwrap_or_default()
        };
        if bytes.len() > left {
            return Ok((chunks, true));
        }
        left -= bytes.len();

        let binary = binary(&bytes);
        let patch = parse::untracked(&file.path, if binary { &[] } else { &bytes });
        let mut chunk = parse::split(&patch)
            .pop()
            .ok_or_else(|| Error::Git(format!("could not read {}", file.path)))?;
        chunk.path = file.path;
        chunk.change = Change::Untracked;
        chunk.binary = binary;
        chunks.push((chunk, large));
    }

    Ok((chunks, false))
}

fn limit(base: Option<String>, chunks: Vec<(parse::Chunk, bool)>) -> Diff {
    let mut budget = TOTAL_LIMIT;
    let mut truncated = false;
    let files = chunks
        .into_iter()
        .map(|(chunk, oversized)| {
            let large = oversized
                || chunk.patch.len() > FILE_LIMIT
                || chunk.added.saturating_add(chunk.removed) > LINE_LIMIT;
            let fits = !large && !chunk.binary && chunk.patch.len() <= budget;
            if fits {
                budget -= chunk.patch.len();
            } else if !large && !chunk.binary {
                truncated = true;
            }

            FileDiff {
                path: chunk.path,
                from: chunk.from,
                change: chunk.change,
                added: chunk.added,
                removed: chunk.removed,
                patch: fits.then_some(chunk.patch),
                binary: chunk.binary,
                large,
            }
        })
        .collect();

    Diff {
        base,
        files,
        truncated,
    }
}

#[cfg(test)]
mod tests;
