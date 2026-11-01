use git::{add_detached, add_worktree, discard_worktree, heads, remove_worktree, worktrees};
use tokio::fs;

mod common;

use common::{repo, run};

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
async fn a_bare_layout_has_no_main_checkout() {
    let dir = tempfile::tempdir().unwrap();
    let source = repo(dir.path(), &[]).await.unwrap();
    run(
        dir.path(),
        &[
            "clone",
            "--quiet",
            "--bare",
            source.to_str().unwrap(),
            "app.git",
        ],
    )
    .await
    .unwrap();
    let bare = dir.path().join("app.git");
    let main = dir.path().join("main");
    let login = dir.path().join("login");
    run(
        &bare,
        &["worktree", "add", "--quiet", main.to_str().unwrap(), "main"],
    )
    .await
    .unwrap();
    run(
        &bare,
        &[
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

    let listed = worktrees(&main).await.unwrap();

    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|worktree| !worktree.main));
    assert!(worktrees(&source).await.unwrap()[0].main);
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

#[tokio::test]
async fn adds_a_detached_worktree_and_discards_it() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo(dir.path(), &[]).await.unwrap();
    let review = dir.path().join("worktrees").join("pr-42");

    add_detached(&repo, &review).await.unwrap();

    let listed = worktrees(&repo).await.unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[1].branch, None);

    discard_worktree(&repo, &review).await;

    assert_eq!(worktrees(&repo).await.unwrap().len(), 1);
    assert!(!review.exists());
}
