use git::{add_worktree, removable, remove_worktree};
use tokio::fs;

mod common;

use common::{repo, run};

#[tokio::test]
async fn refuses_a_locked_worktree_before_anything_else_happens() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    run(&repo, &["worktree", "lock", worktree.to_str().unwrap()])
        .await
        .unwrap();

    let error = removable(&repo, &worktree).await.unwrap_err();

    assert!(error.to_string().contains("locked"), "{error}");
    assert!(remove_worktree(&repo, &worktree).await.is_err());
    assert!(worktree.exists());
}

#[tokio::test]
async fn refuses_a_worktree_with_submodules_before_anything_else_happens() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let library = dir.path().join("library");
    fs::create_dir_all(&library).await.unwrap();
    run(&library, &["init", "--quiet", "--initial-branch", "main"])
        .await
        .unwrap();
    run(
        &library,
        &["commit", "--quiet", "--allow-empty", "-m", "first"],
    )
    .await
    .unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    run(
        &worktree,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "--quiet",
            "add",
            library.to_str().unwrap(),
            "lib",
        ],
    )
    .await
    .unwrap();
    run(&worktree, &["commit", "--quiet", "-m", "library"])
        .await
        .unwrap();

    let error = removable(&repo, &worktree).await.unwrap_err();

    assert!(error.to_string().contains("submodules"), "{error}");
    assert!(remove_worktree(&repo, &worktree).await.is_err());
    assert!(worktree.join("lib").exists());
}

#[tokio::test]
async fn refuses_to_remove_a_folder_that_is_not_a_worktree_of_the_repo() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let other = dir.path().join("other");
    fs::create_dir_all(&other).await.unwrap();
    run(&other, &["init", "--quiet", "--initial-branch", "main"])
        .await
        .unwrap();
    run(
        &other,
        &["commit", "--quiet", "--allow-empty", "-m", "first"],
    )
    .await
    .unwrap();

    assert!(remove_worktree(&repo, &other).await.is_err());
    assert!(remove_worktree(&repo, &repo).await.is_err());

    assert!(other.join(".git").exists());
    assert!(repo.join(".git").exists());
}

#[tokio::test]
async fn refuses_to_remove_a_worktree_in_the_middle_of_a_bisect() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    run(&worktree, &["bisect", "start"]).await.unwrap();

    let error = removable(&repo, &worktree).await.unwrap_err();

    assert!(error.to_string().contains("bisect"), "{error}");
}
