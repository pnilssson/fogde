use std::path::{Path, PathBuf};

use super::*;
use crate::branches::branches;
use crate::commit::commit;
use crate::testing::{repo, write};
use crate::write::push;

async fn merged(dir: &Path) -> (PathBuf, String) {
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
    tokio::fs::create_dir(&app).await.unwrap();
    repo(&app).await;
    git(&app, ["remote", "add", "origin", remote.to_str().unwrap()])
        .await
        .unwrap();
    write(&app, "app.txt", "one\n").await;
    commit(&app, &["app.txt".to_owned()], "start")
        .await
        .unwrap();
    push(&app).await.unwrap();

    git(&app, ["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();
    write(&app, "login.txt", "login\n").await;
    commit(&app, &["login.txt".to_owned()], "Add login")
        .await
        .unwrap();
    push(&app).await.unwrap();
    let head = git(&app, ["rev-parse", "HEAD"]).await.unwrap();
    let head = String::from_utf8_lossy(&head).trim().to_owned();

    git(
        &remote,
        ["update-ref", "refs/heads/main", "refs/heads/feature/login"],
    )
    .await
    .unwrap();

    (app, head)
}

#[tokio::test]
async fn refuses_a_checkout_with_changes() {
    let dir = tempfile::tempdir().unwrap();
    let (app, head) = merged(dir.path()).await;
    write(&app, "notes.txt", "draft\n").await;

    let error = landed(&app, &head, "main").await.unwrap_err();

    assert!(error.to_string().contains("commit or discard"), "{error}");
}

#[tokio::test]
async fn refuses_a_branch_with_commits_after_the_pull_request() {
    let dir = tempfile::tempdir().unwrap();
    let (app, head) = merged(dir.path()).await;
    write(&app, "more.txt", "more\n").await;
    commit(&app, &["more.txt".to_owned()], "More")
        .await
        .unwrap();

    let error = landed(&app, &head, "main").await.unwrap_err();

    assert!(
        error.to_string().contains("not in the pull request"),
        "{error}"
    );
}

#[tokio::test]
async fn refuses_the_base_branch() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = merged(dir.path()).await;
    git(&app, ["switch", "--quiet", "main"]).await.unwrap();
    git(&app, ["pull", "--quiet", "--ff-only"]).await.unwrap();
    let head = git(&app, ["rev-parse", "HEAD"]).await.unwrap();

    let error = landed(&app, String::from_utf8_lossy(&head).trim(), "main")
        .await
        .unwrap_err();

    assert!(error.to_string().contains("already on main"), "{error}");
}

#[tokio::test]
async fn leaves_a_merged_branch_for_the_updated_base() {
    let dir = tempfile::tempdir().unwrap();
    let (app, head) = merged(dir.path()).await;

    let branch = landed(&app, &head, "main").await.unwrap();
    leave(&app, &branch, "main").await.unwrap();

    let now = status(&app).await.unwrap();
    assert_eq!(now.branch.as_deref(), Some("main"));
    assert_eq!(now.behind, 0);
    assert!(app.join("login.txt").exists());
    assert!(
        !branches(&app)
            .await
            .unwrap()
            .iter()
            .any(|branch| branch.name == "feature/login" && !branch.remote)
    );
}
