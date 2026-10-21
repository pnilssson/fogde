use std::path::{Path, PathBuf};

use tokio::fs;

use crate::run::git;

const SHORT: usize = 7;

#[derive(Debug, PartialEq, Eq)]
enum Head {
    Branch(String),
    Detached(String),
    Reftable,
}

pub async fn branch(folder: &Path) -> Option<String> {
    let (root, dir) = repo(folder).await?;
    let head = fs::read_to_string(dir.join("HEAD")).await.ok()?;

    match parse(&head)? {
        Head::Branch(name) => Some(name),
        Head::Detached(commit) => Some(commit),
        Head::Reftable => reftable(&root).await,
    }
}

pub async fn heads(folder: &Path) -> Vec<(String, String)> {
    let Some((_, dir)) = repo(folder).await else {
        return Vec::new();
    };
    let common = match fs::read_to_string(dir.join("commondir")).await {
        Ok(link) => dir.join(link.trim()),
        Err(_) => dir,
    };

    let mut heads = vec![(String::new(), head(&common).await)];
    if let Ok(mut entries) = fs::read_dir(common.join("worktrees")).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            heads.push((name, head(&entry.path()).await));
        }
    }
    heads.sort();

    heads
}

async fn head(dir: &Path) -> String {
    let head = fs::read_to_string(dir.join("HEAD"))
        .await
        .unwrap_or_default();
    let tables = fs::read_to_string(dir.join("reftable").join("tables.list"))
        .await
        .unwrap_or_default();

    head + &tables
}

async fn repo(folder: &Path) -> Option<(PathBuf, PathBuf)> {
    for dir in folder.ancestors() {
        if let Some(git_dir) = git_dir(dir).await {
            return Some((dir.to_owned(), git_dir));
        }
    }

    None
}

async fn git_dir(dir: &Path) -> Option<PathBuf> {
    let dot = dir.join(".git");
    let metadata = fs::metadata(&dot).await.ok()?;
    if metadata.is_dir() {
        return Some(dot);
    }

    let link = fs::read_to_string(&dot).await.ok()?;
    let target = link.trim().strip_prefix("gitdir:")?.trim();

    Some(dir.join(target))
}

fn parse(head: &str) -> Option<Head> {
    let head = head.trim();
    if let Some(target) = head.strip_prefix("ref:") {
        let target = target.trim();
        if target == "refs/heads/.invalid" {
            return Some(Head::Reftable);
        }

        let name = target.strip_prefix("refs/heads/").unwrap_or(target);
        return Some(Head::Branch(name.to_owned()));
    }

    let commit = head.get(..SHORT)?;
    commit
        .chars()
        .all(|c| c.is_ascii_hexdigit())
        .then(|| Head::Detached(commit.to_owned()))
}

async fn reftable(root: &Path) -> Option<String> {
    let named = git(root, ["symbolic-ref", "--short", "HEAD"]).await;
    let output = match named {
        Ok(output) => output,
        Err(_) => git(root, ["rev-parse", "--short", "HEAD"]).await.ok()?,
    };
    let name = String::from_utf8_lossy(&output).trim().to_owned();

    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).await.unwrap();
        fs::write(path, text).await.unwrap();
    }

    #[test]
    fn reads_branches_commits_and_reftable_markers() {
        assert_eq!(
            parse("ref: refs/heads/main\n"),
            Some(Head::Branch("main".to_owned()))
        );
        assert_eq!(
            parse("ref: refs/heads/feature/login"),
            Some(Head::Branch("feature/login".to_owned()))
        );
        assert_eq!(
            parse("3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345\n"),
            Some(Head::Detached("3f2a9b1".to_owned()))
        );
        assert_eq!(parse("ref: refs/heads/.invalid"), Some(Head::Reftable));
        assert_eq!(parse("not a head"), None);
        assert_eq!(parse(""), None);
    }

    #[tokio::test]
    async fn finds_the_branch_from_any_folder_inside_a_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("app");
        write(&repo.join(".git").join("HEAD"), "ref: refs/heads/main\n").await;
        fs::create_dir_all(repo.join("src").join("deep"))
            .await
            .unwrap();

        assert_eq!(branch(&repo).await.as_deref(), Some("main"));
        assert_eq!(
            branch(&repo.join("src").join("deep")).await.as_deref(),
            Some("main")
        );
        assert_eq!(branch(dir.path()).await, None);
    }

    #[tokio::test]
    async fn follows_a_linked_worktree_to_its_own_head() {
        let dir = tempfile::tempdir().unwrap();
        let admin = dir
            .path()
            .join("app")
            .join(".git")
            .join("worktrees")
            .join("login");
        write(&admin.join("HEAD"), "ref: refs/heads/feature/login\n").await;
        let worktree = dir.path().join("login");
        write(
            &worktree.join(".git"),
            &format!("gitdir: {}\n", admin.display()),
        )
        .await;

        assert_eq!(branch(&worktree).await.as_deref(), Some("feature/login"));
    }

    #[tokio::test]
    async fn reads_a_real_repository() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), ["init", "--quiet", "--initial-branch", "trunk"])
            .await
            .unwrap();

        assert_eq!(branch(dir.path()).await.as_deref(), Some("trunk"));
    }
}
