use std::path::{Path, PathBuf};

use tokio::fs;

use super::*;
use crate::commit::commit;
use crate::run::git;
use crate::testing::{log, repo, write};

async fn published(dir: &Path) -> (PathBuf, PathBuf) {
    let remote = dir.join("remote.git");
    let app = dir.join("app");
    git(
        dir,
        [
            "init",
            "--quiet",
            "--bare",
            "--initial-branch",
            "main",
            "remote.git",
        ],
    )
    .await
    .unwrap();
    fs::create_dir(&app).await.unwrap();
    repo(&app).await;
    git(&app, ["remote", "add", "origin", remote.to_str().unwrap()])
        .await
        .unwrap();
    write(&app, "app.txt", "one\n").await;
    commit(&app, &["app.txt".to_owned()], "start")
        .await
        .unwrap();
    push(&app).await.unwrap();

    (app, remote)
}

async fn cloned(dir: &Path, remote: &Path) -> PathBuf {
    git(dir, ["clone", "--quiet", remote.to_str().unwrap(), "other"])
        .await
        .unwrap();
    let other = dir.join("other");
    git(&other, ["config", "user.name", "Test"]).await.unwrap();
    git(&other, ["config", "user.email", "test@example.com"])
        .await
        .unwrap();

    other
}

#[tokio::test]
async fn the_first_push_sets_the_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;

    let pushed = status(&app).await.unwrap();

    assert_eq!(pushed.upstream.as_deref(), Some("origin/main"));
    assert_eq!((pushed.ahead, pushed.behind), (0, 0));
}

#[tokio::test]
async fn pulls_a_fast_forward_and_refuses_a_diverged_branch() {
    let dir = tempfile::tempdir().unwrap();
    let (app, remote) = published(dir.path()).await;
    let other = cloned(dir.path(), &remote).await;
    write(&other, "other.txt", "other\n").await;
    commit(&other, &["other.txt".to_owned()], "From other")
        .await
        .unwrap();
    push(&other).await.unwrap();

    pull(&app).await.unwrap();

    assert_eq!(log(&app).await, ["From other", "start"]);

    write(&other, "other.txt", "changed\n").await;
    commit(&other, &["other.txt".to_owned()], "Again")
        .await
        .unwrap();
    push(&other).await.unwrap();
    write(&app, "app.txt", "local\n").await;
    commit(&app, &["app.txt".to_owned()], "Local")
        .await
        .unwrap();

    let error = pull(&app).await.unwrap_err();

    assert!(error.to_string().contains("diverged"), "{error}");
    assert_eq!(log(&app).await, ["Local", "From other", "start"]);
}

#[tokio::test]
async fn switches_branches_unless_changes_would_be_lost() {
    let dir = tempfile::tempdir().unwrap();
    let (app, remote) = published(dir.path()).await;
    let other = cloned(dir.path(), &remote).await;
    git(&other, ["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();
    write(&other, "app.txt", "login\n").await;
    commit(&other, &["app.txt".to_owned()], "Login")
        .await
        .unwrap();
    push(&other).await.unwrap();
    git(&app, ["fetch", "--quiet"]).await.unwrap();

    assert!(
        crate::branches(&app)
            .await
            .unwrap()
            .iter()
            .any(|branch| branch.name == "feature/login" && branch.remote)
    );
    assert!(switch(&app, "--orphan").await.is_err());

    write(&app, "app.txt", "unsaved\n").await;
    assert!(switch(&app, "feature/login").await.is_err());

    git(&app, ["checkout", "--quiet", "--", "app.txt"])
        .await
        .unwrap();
    switch(&app, "feature/login").await.unwrap();

    let switched = status(&app).await.unwrap();
    assert_eq!(switched.branch.as_deref(), Some("feature/login"));
    assert_eq!(switched.upstream.as_deref(), Some("origin/feature/login"));
}

#[tokio::test]
async fn fetching_counts_incoming_commits_and_records_the_time() {
    let dir = tempfile::tempdir().unwrap();
    let (app, remote) = published(dir.path()).await;
    let other = cloned(dir.path(), &remote).await;
    write(&other, "other.txt", "other\n").await;
    commit(&other, &["other.txt".to_owned()], "From other")
        .await
        .unwrap();
    push(&other).await.unwrap();

    refresh(&app).await.unwrap();

    assert_eq!(status(&app).await.unwrap().behind, 1);
    assert!(fetched(&app).await.is_some());
}

#[tokio::test]
async fn records_the_fetch_time_of_a_linked_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;
    let login = dir.path().join("login");
    git(
        &app,
        [
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature/login",
            login.to_str().unwrap(),
        ],
    )
    .await
    .unwrap();
    assert!(fetched(&login).await.is_none());

    refresh(&login).await.unwrap();

    assert!(fetched(&login).await.is_some());
}

#[tokio::test]
async fn a_fetch_in_one_checkout_counts_for_its_worktrees() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;
    let login = dir.path().join("login");
    git(
        &app,
        [
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature/login",
            login.to_str().unwrap(),
        ],
    )
    .await
    .unwrap();

    refresh(&app).await.unwrap();

    assert!(fetched(&login).await.is_some());
}

#[tokio::test]
async fn fetching_without_a_remote_says_so() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;

    let error = refresh(dir.path()).await.unwrap_err();

    assert!(error.to_string().contains("add a remote"), "{error}");
    assert!(fetched(dir.path()).await.is_none());
}

#[tokio::test]
async fn the_first_push_goes_to_the_remote_git_pushes_to() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;
    let fork = dir.path().join("fork.git");
    git(dir.path(), ["init", "--quiet", "--bare", "fork.git"])
        .await
        .unwrap();
    git(&app, ["remote", "add", "fork", fork.to_str().unwrap()])
        .await
        .unwrap();
    git(&app, ["config", "remote.pushDefault", "fork"])
        .await
        .unwrap();
    git(&app, ["switch", "--quiet", "--create", "feature/login"])
        .await
        .unwrap();

    push(&app).await.unwrap();

    let pushed = status(&app).await.unwrap();
    assert_eq!(pushed.upstream.as_deref(), Some("fork/feature/login"));
}

#[tokio::test]
async fn a_new_branch_takes_the_uncommitted_changes_along() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;
    write(&app, "login.txt", "login\n").await;

    branch_off(&app, "feature/login").await.unwrap();

    let now = status(&app).await.unwrap();
    assert_eq!(now.branch.as_deref(), Some("feature/login"));
    assert_eq!(now.upstream, None);
    assert_eq!(now.files.len(), 1);
}

#[tokio::test]
async fn refuses_names_git_cannot_use_for_a_branch() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = published(dir.path()).await;

    for name in ["", "-d", "feature..login", "main"] {
        assert!(branch_off(&app, name).await.is_err(), "{name:?}");
    }
    assert_eq!(status(&app).await.unwrap().branch.as_deref(), Some("main"));
}
