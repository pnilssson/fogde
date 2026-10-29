use tokio::fs;

use super::*;
use crate::status::File;
use crate::testing::{log, repo, write};

#[tokio::test]
async fn commits_only_the_chosen_files() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "a.txt", "a\n").await;
    write(dir.path(), "[a].txt", "brackets\n").await;
    write(dir.path(), "staged.txt", "staged\n").await;
    git(dir.path(), ["add", "staged.txt"]).await.unwrap();

    commit(dir.path(), &["[a].txt".to_owned()], "Add brackets")
        .await
        .unwrap();

    assert_eq!(log(dir.path()).await, ["Add brackets"]);
    let files = status(dir.path()).await.unwrap().files;
    assert!(files.contains(&File {
        path: "a.txt".to_owned(),
        from: None,
        change: Change::Untracked,
    }));
    assert!(files.contains(&File {
        path: "staged.txt".to_owned(),
        from: None,
        change: Change::Added,
    }));
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn commits_deletions_and_refuses_an_empty_message() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "old.txt", "old\n").await;
    commit(dir.path(), &["old.txt".to_owned()], "Add")
        .await
        .unwrap();
    fs::remove_file(dir.path().join("old.txt")).await.unwrap();

    assert!(
        commit(dir.path(), &["old.txt".to_owned()], "  ")
            .await
            .is_err()
    );
    assert!(commit(dir.path(), &[], "Remove").await.is_err());
    commit(dir.path(), &["old.txt".to_owned()], "Remove")
        .await
        .unwrap();

    assert_eq!(log(dir.path()).await, ["Remove", "Add"]);
    assert_eq!(status(dir.path()).await.unwrap().files, Vec::<File>::new());
}

#[tokio::test]
async fn commits_staged_renames_and_removals() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "old.txt", "old\n").await;
    write(dir.path(), "gone.txt", "gone\n").await;
    commit(
        dir.path(),
        &["old.txt".to_owned(), "gone.txt".to_owned()],
        "Add",
    )
    .await
    .unwrap();
    git(dir.path(), ["mv", "old.txt", "new.txt"]).await.unwrap();
    git(dir.path(), ["rm", "--quiet", "gone.txt"])
        .await
        .unwrap();

    commit(
        dir.path(),
        &[
            "old.txt".to_owned(),
            "new.txt".to_owned(),
            "gone.txt".to_owned(),
        ],
        "Rename",
    )
    .await
    .unwrap();

    assert_eq!(log(dir.path()).await, ["Rename", "Add"]);
    assert_eq!(status(dir.path()).await.unwrap().files, Vec::<File>::new());
}

#[tokio::test]
async fn refuses_to_commit_during_a_merge_without_staging_conflicts() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "app.txt", "one\n").await;
    commit(dir.path(), &["app.txt".to_owned()], "Start")
        .await
        .unwrap();
    git(dir.path(), ["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();
    write(dir.path(), "app.txt", "login\n").await;
    commit(dir.path(), &["app.txt".to_owned()], "Login")
        .await
        .unwrap();
    git(dir.path(), ["switch", "--quiet", "main"])
        .await
        .unwrap();
    write(dir.path(), "app.txt", "main\n").await;
    commit(dir.path(), &["app.txt".to_owned()], "Main")
        .await
        .unwrap();
    assert!(
        git(dir.path(), ["merge", "--quiet", "feature/login"])
            .await
            .is_err()
    );

    let error = commit(dir.path(), &["app.txt".to_owned()], "Merge")
        .await
        .unwrap_err();

    assert!(error.to_string().contains("merge"), "{error}");
    assert_eq!(
        status(dir.path()).await.unwrap().files,
        [File {
            path: "app.txt".to_owned(),
            from: None,
            change: Change::Conflicted,
        }]
    );
}

#[tokio::test]
async fn unstages_new_files_when_the_commit_fails() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "app.txt", "one\n").await;
    commit(dir.path(), &["app.txt".to_owned()], "Start")
        .await
        .unwrap();
    git(dir.path(), ["config", "commit.gpgsign", "true"])
        .await
        .unwrap();
    git(dir.path(), ["config", "gpg.program", "missing-signer"])
        .await
        .unwrap();
    write(dir.path(), "app.txt", "two\n").await;
    write(dir.path(), "login.txt", "login\n").await;

    let paths = ["app.txt".to_owned(), "login.txt".to_owned()];
    assert!(commit(dir.path(), &paths, "Login").await.is_err());

    assert_eq!(log(dir.path()).await, ["Start"]);
    let files = status(dir.path()).await.unwrap().files;
    assert!(files.contains(&File {
        path: "login.txt".to_owned(),
        from: None,
        change: Change::Untracked,
    }));
    assert!(files.contains(&File {
        path: "app.txt".to_owned(),
        from: None,
        change: Change::Modified,
    }));
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn unstages_new_files_a_failing_hook_rewrote() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "app.txt", "one\n").await;
    commit(dir.path(), &["app.txt".to_owned()], "Start")
        .await
        .unwrap();
    let hooks = dir.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks).await.unwrap();
    let hook = hooks.join("pre-commit");
    fs::write(&hook, "#!/bin/sh\necho formatted > login.txt\nexit 1\n")
        .await
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .await
            .unwrap();
    }
    write(dir.path(), "login.txt", "login\n").await;

    assert!(
        commit(dir.path(), &["login.txt".to_owned()], "Login")
            .await
            .is_err()
    );

    assert_eq!(
        status(dir.path()).await.unwrap().files,
        [File {
            path: "login.txt".to_owned(),
            from: None,
            change: Change::Untracked,
        }]
    );
}
