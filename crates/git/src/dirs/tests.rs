use super::*;
use crate::run::git;

const NAMES: [&str; 14] = [
    "HEAD",
    "index",
    "logs/HEAD",
    "FETCH_HEAD",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
    "locked",
    "modules",
    "objects",
    "packed-refs",
    "reftable/tables.list",
];

async fn asked(folder: &Path, name: &str) -> PathBuf {
    let output = git(
        folder,
        ["rev-parse", "--path-format=absolute", "--git-path", name],
    )
    .await
    .unwrap();
    let path = String::from_utf8(output).unwrap();

    normal(Path::new(path.trim()))
}

fn normal(path: &Path) -> PathBuf {
    let text = path.to_string_lossy().replace('\\', "/");

    std::fs::canonicalize(&text)
        .ok()
        .or_else(|| {
            let parent = Path::new(&text).parent()?;
            let name = Path::new(&text).file_name()?;
            Some(normal(parent).join(name))
        })
        .unwrap_or_else(|| PathBuf::from(text))
}

async fn checked(folder: &Path) {
    let dirs = Dirs::find(folder).await.unwrap();
    for name in NAMES {
        assert_eq!(
            normal(&dirs.path(name)),
            asked(folder, name).await,
            "{name}"
        );
    }
}

#[tokio::test]
async fn finds_the_same_files_git_does_in_a_repo_and_a_linked_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("app");
    let worktree = dir.path().join("login");
    git(dir.path(), ["init", "--quiet", "app"]).await.unwrap();
    git(
        &repo,
        [
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "start",
        ],
    )
    .await
    .unwrap();
    git(
        &repo,
        [
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature/login",
            worktree.to_str().unwrap(),
        ],
    )
    .await
    .unwrap();

    checked(&repo).await;
    checked(&worktree).await;
}

#[tokio::test]
async fn stamps_change_when_the_index_or_head_moves() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), ["init", "--quiet"]).await.unwrap();
    let before = stamp(dir.path()).await.unwrap();

    tokio::fs::write(dir.path().join("app.txt"), "one\n")
        .await
        .unwrap();

    assert_eq!(stamp(dir.path()).await.unwrap(), before);

    git(dir.path(), ["add", "app.txt"]).await.unwrap();

    assert_ne!(stamp(dir.path()).await.unwrap(), before);
}
