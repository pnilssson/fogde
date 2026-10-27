use std::path::{Path, PathBuf};

use git::{add_worktree, clean, heads, remove_merged_branch, remove_worktree, worktrees};
use tokio::fs;
use tokio::process::Command;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn run(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .output()
        .await?;
    if !output.status.success() {
        return Err(format!("git {args:?} failed").into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

async fn repo(root: &Path, files: &[(&str, &str)]) -> Result<PathBuf> {
    let repo = root.join("repo");
    fs::create_dir_all(&repo).await?;
    run(&repo, &["init", "--quiet", "--initial-branch", "main"]).await?;
    fs::write(repo.join(".gitignore"), ".env\nbuild/\n").await?;
    run(&repo, &["add", ".gitignore"]).await?;
    run(&repo, &["commit", "--quiet", "-m", "first"]).await?;

    for (file, text) in files {
        let path = repo.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, text).await?;
    }

    Ok(repo)
}

#[tokio::test]
async fn copies_ignored_files_listed_in_worktreeinclude() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(
        dir.path(),
        &[
            (".env", "SECRET=1"),
            ("build/cache.txt", "cache"),
            ("notes.txt", "untracked but not ignored"),
            (".worktreeinclude", ".env\nnotes.txt\n"),
        ],
    )
    .await
    .unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    assert_eq!(
        fs::read_to_string(worktree.join(".env")).await.unwrap(),
        "SECRET=1"
    );
    assert!(!worktree.join("notes.txt").exists());
    assert!(!worktree.join("build").exists());
    let listed = worktrees(&repo).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[1].branch.as_deref(), Some("feature/login"));

    remove_worktree(&repo, &worktree).await.unwrap();

    assert!(!worktree.exists());
    assert_eq!(worktrees(&repo).await.unwrap().len(), 1);
}

#[tokio::test]
async fn copies_the_env_file_without_a_worktreeinclude() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[(".env", "SECRET=1")]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    assert!(worktree.join(".env").exists());
}

#[tokio::test]
async fn skips_nested_repositories() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(
        dir.path(),
        &[(".env", "SECRET=1"), (".worktreeinclude", ".env\nbuild/\n")],
    )
    .await
    .unwrap();
    let nested = repo.join("build").join("lib");
    fs::create_dir_all(&nested).await.unwrap();
    run(&nested, &["init", "--quiet"]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    assert!(worktree.join(".env").exists());
    assert!(!worktree.join("build").exists());
}

#[tokio::test]
async fn reuses_a_name_after_its_worktree_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    remove_worktree(&repo, &worktree).await.unwrap();
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    assert!(worktree.exists());
}

#[tokio::test]
async fn a_worktree_git_refuses_to_remove_is_not_clean() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    assert!(clean(&worktree).await.unwrap());

    fs::write(worktree.join("draft.txt"), "draft")
        .await
        .unwrap();

    assert!(!clean(&worktree).await.unwrap());
    assert!(remove_worktree(&repo, &worktree).await.is_err());
}

#[tokio::test]
async fn removes_a_merged_branch() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();

    let branch = remove_worktree(&repo, &worktree).await.unwrap();

    assert_eq!(branch.as_deref(), Some("feature/login"));
    assert!(remove_merged_branch(&repo, "feature/login").await.unwrap());
    assert_eq!(
        run(&repo, &["branch", "--list", "feature/login"])
            .await
            .unwrap(),
        ""
    );
}

#[tokio::test]
async fn keeps_a_branch_with_new_commits_and_reuses_it() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    run(
        &worktree,
        &["commit", "--quiet", "--allow-empty", "-m", "work"],
    )
    .await
    .unwrap();

    remove_worktree(&repo, &worktree).await.unwrap();

    assert!(!remove_merged_branch(&repo, "feature/login").await.unwrap());
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    assert_eq!(
        run(&worktree, &["log", "-1", "--format=%s"]).await.unwrap(),
        "work\n"
    );
}

#[tokio::test]
async fn reports_a_folder_that_is_not_a_repo() {
    let dir = tempfile::tempdir().unwrap();

    assert!(worktrees(dir.path()).await.is_err());
}

#[tokio::test]
async fn heads_change_when_worktrees_come_go_or_switch_branch() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    let before = heads(&repo).await;

    run(
        &repo,
        &[
            "worktree",
            "add",
            "-b",
            "feature/login",
            worktree.to_str().unwrap(),
        ],
    )
    .await
    .unwrap();
    let added = heads(&repo).await;

    assert_ne!(added, before);
    assert_eq!(heads(&worktree).await, added);

    run(&worktree, &["switch", "--quiet", "-c", "feature/signup"])
        .await
        .unwrap();
    let switched = heads(&repo).await;

    assert_ne!(switched, added);

    run(&repo, &["worktree", "remove", worktree.to_str().unwrap()])
        .await
        .unwrap();

    assert_eq!(heads(&repo).await, before);
    assert_eq!(heads(dir.path()).await, Vec::new());
}

#[tokio::test]
async fn heads_follow_branch_switches_in_reftable_repos() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("app");
    fs::create_dir_all(&repo).await.unwrap();
    run(
        &repo,
        &[
            "init",
            "--quiet",
            "--ref-format=reftable",
            "--initial-branch",
            "main",
        ],
    )
    .await
    .unwrap();
    run(
        &repo,
        &["commit", "--quiet", "--allow-empty", "-m", "first"],
    )
    .await
    .unwrap();
    let before = heads(&repo).await;

    run(&repo, &["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();

    assert_ne!(heads(&repo).await, before);
}
