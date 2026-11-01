use git::{add_detached, add_worktree, clean, remove_merged_branch, remove_worktree};
use tokio::fs;

mod common;

use common::{repo, run};

#[tokio::test]
async fn reuses_a_name_after_its_worktree_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    remove_worktree(&repo, &worktree).await.unwrap();
    assert!(remove_merged_branch(&repo, "feature/login").await.unwrap());
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
async fn keeps_a_branch_with_new_commits_and_refuses_to_reuse_it() {
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
    let error = add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("already exists"), "{error}");
    assert!(!worktree.exists());
    assert_eq!(
        run(&repo, &["log", "-1", "--format=%s", "feature/login"])
            .await
            .unwrap(),
        "work\n"
    );
}

#[tokio::test]
async fn refuses_to_remove_commits_no_branch_contains() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let review = dir.path().join("worktrees").join("pr-42");
    add_detached(&repo, &review).await.unwrap();
    run(
        &review,
        &["commit", "--quiet", "--allow-empty", "-m", "loose"],
    )
    .await
    .unwrap();

    let error = remove_worktree(&repo, &review).await.unwrap_err();

    assert!(error.to_string().contains("branch"), "{error}");
    assert!(review.exists());
}

#[tokio::test]
async fn removes_a_detached_worktree_whose_commit_a_branch_contains() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let review = dir.path().join("worktrees").join("pr-42");
    add_detached(&repo, &review).await.unwrap();

    remove_worktree(&repo, &review).await.unwrap();

    assert!(!review.exists());
}

#[tokio::test]
async fn refuses_to_remove_a_worktree_in_the_middle_of_a_rebase() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    fs::write(worktree.join("app.txt"), "login\n")
        .await
        .unwrap();
    run(&worktree, &["add", "app.txt"]).await.unwrap();
    run(&worktree, &["commit", "--quiet", "-m", "login"])
        .await
        .unwrap();
    fs::write(repo.join("app.txt"), "main\n").await.unwrap();
    run(&repo, &["add", "app.txt"]).await.unwrap();
    run(&repo, &["commit", "--quiet", "-m", "main"])
        .await
        .unwrap();
    assert!(
        run(&worktree, &["rebase", "--quiet", "main"])
            .await
            .is_err()
    );
    fs::write(worktree.join("app.txt"), "main\n").await.unwrap();
    run(&worktree, &["add", "app.txt"]).await.unwrap();

    let error = remove_worktree(&repo, &worktree).await.unwrap_err();

    assert!(error.to_string().contains("rebase"), "{error}");
    assert!(worktree.exists());
}

#[tokio::test]
async fn refuses_to_reuse_a_branch_that_already_exists() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    run(&repo, &["branch", "feature/login"]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");

    let error = add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap_err();

    assert!(error.to_string().contains("already exists"), "{error}");
    assert!(!worktree.exists());
}

#[cfg(windows)]
#[tokio::test]
async fn removes_a_worktree_once_a_closing_shell_lets_go_of_it() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let worktree = dir.path().join("worktrees").join("login");
    add_worktree(&repo, &worktree, "feature/login")
        .await
        .unwrap();
    let mut shell = tokio::process::Command::new("ping")
        .args(["-n", "30", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .current_dir(&worktree)
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let closing = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        shell.kill().await.unwrap();
    });

    let removed = remove_worktree(&repo, &worktree).await;
    closing.await.unwrap();

    assert_eq!(removed.unwrap().as_deref(), Some("feature/login"));
    assert!(!worktree.exists());
    assert_eq!(git::worktrees(&repo).await.unwrap().len(), 1);
}
