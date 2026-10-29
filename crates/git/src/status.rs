use std::path::Path;

use crate::Error;
use crate::run::git;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<File>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub from: Option<String>,
    pub change: Change,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
}

pub async fn status(root: &Path) -> Result<Status, Error> {
    let output = git(
        root,
        [
            "--no-optional-locks",
            "status",
            "--porcelain=v2",
            "--branch",
            "--untracked-files=all",
            "-z",
        ],
    )
    .await?;

    Ok(parse(&String::from_utf8_lossy(&output)))
}

fn parse(output: &str) -> Status {
    let mut status = Status::default();
    let mut entries = output.split('\0');

    while let Some(entry) = entries.next() {
        if let Some(header) = entry.strip_prefix("# ") {
            branch(&mut status, header);
            continue;
        }

        let file = match entry.split_at_checked(2) {
            Some(("1 ", rest)) => ordinary(rest, 7),
            Some(("2 ", rest)) => ordinary(rest, 8).map(|file| File {
                from: entries.next().map(str::to_owned),
                ..file
            }),
            Some(("u ", rest)) => conflicted(rest),
            Some(("? ", path)) => Some(File {
                path: path.to_owned(),
                from: None,
                change: Change::Untracked,
            }),
            _ => None,
        };
        status.files.extend(file);
    }

    status
}

fn branch(status: &mut Status, header: &str) {
    let Some((key, value)) = header.split_once(' ') else {
        return;
    };

    match key {
        "branch.head" if value != "(detached)" => status.branch = Some(value.to_owned()),
        "branch.upstream" => status.upstream = Some(value.to_owned()),
        "branch.ab" => {
            let mut counts = value.split(' ');
            status.ahead = count(counts.next(), '+');
            status.behind = count(counts.next(), '-');
        }
        _ => {}
    }
}

fn count(value: Option<&str>, sign: char) -> u32 {
    value
        .and_then(|value| value.strip_prefix(sign))
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn ordinary(rest: &str, fields: usize) -> Option<File> {
    let mut parts = rest.splitn(fields + 1, ' ');
    let codes = parts.next()?;
    let path = parts.nth(fields - 1)?;

    Some(File {
        path: path.to_owned(),
        from: None,
        change: change(codes),
    })
}

fn conflicted(rest: &str) -> Option<File> {
    let path = rest.splitn(10, ' ').nth(9)?;

    Some(File {
        path: path.to_owned(),
        from: None,
        change: Change::Conflicted,
    })
}

fn change(codes: &str) -> Change {
    if codes.ends_with('D') {
        return Change::Deleted;
    }

    let mut letters = codes.chars().filter(|&c| c != '.');
    match letters.next() {
        Some('A') => Change::Added,
        Some('D') => Change::Deleted,
        Some('R') => Change::Renamed,
        Some('C') => Change::Copied,
        _ => Change::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, change: Change) -> File {
        File {
            path: path.to_owned(),
            from: None,
            change,
        }
    }

    #[test]
    fn reads_branch_headers_and_every_kind_of_entry() {
        let output = "# branch.oid 3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345\0\
            # branch.head feature/login\0\
            # branch.upstream origin/feature/login\0\
            # branch.ab +2 -3\0\
            1 .M N... 100644 100644 100644 1111111111111111111111111111111111111111 1111111111111111111111111111111111111111 src/app.rs\0\
            1 A. N... 000000 100644 100644 0000000000000000000000000000000000000000 2222222222222222222222222222222222222222 docs/new file.md\0\
            1 MD N... 100644 100644 000000 3333333333333333333333333333333333333333 3333333333333333333333333333333333333333 staged-then-deleted.txt\0\
            1 .D N... 100644 100644 000000 3333333333333333333333333333333333333333 3333333333333333333333333333333333333333 old.txt\0\
            2 R. N... 100644 100644 100644 4444444444444444444444444444444444444444 4444444444444444444444444444444444444444 R100 src/login.rs\0src/auth.rs\0\
            u UU N... 100644 100644 100644 100644 5555555555555555555555555555555555555555 6666666666666666666666666666666666666666 7777777777777777777777777777777777777777 Cargo.lock\0\
            ? notes/todo.md\0";

        let status = parse(output);

        assert_eq!(status.branch.as_deref(), Some("feature/login"));
        assert_eq!(status.upstream.as_deref(), Some("origin/feature/login"));
        assert_eq!((status.ahead, status.behind), (2, 3));
        assert_eq!(
            status.files,
            [
                file("src/app.rs", Change::Modified),
                file("docs/new file.md", Change::Added),
                file("staged-then-deleted.txt", Change::Deleted),
                file("old.txt", Change::Deleted),
                File {
                    from: Some("src/auth.rs".to_owned()),
                    ..file("src/login.rs", Change::Renamed)
                },
                file("Cargo.lock", Change::Conflicted),
                file("notes/todo.md", Change::Untracked),
            ]
        );
    }

    #[test]
    fn reads_a_detached_head_without_upstream() {
        let status = parse("# branch.oid 3f2a9b1\0# branch.head (detached)\0");

        assert_eq!(status, Status::default());
    }

    #[tokio::test]
    async fn reads_a_real_repository() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), ["init", "--quiet", "--initial-branch", "main"])
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("a b.txt"), "one\n")
            .await
            .unwrap();

        let status = status(dir.path()).await.unwrap();

        assert_eq!(status.branch.as_deref(), Some("main"));
        assert_eq!(status.files, [file("a b.txt", Change::Untracked)]);
    }
}
