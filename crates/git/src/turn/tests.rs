use super::*;
use crate::commit::commit;
use crate::run::git;
use crate::status::Change;
use crate::testing::{repo, write};

#[tokio::test]
async fn shows_only_what_changed_since_the_turn_started() {
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "a.txt", "one\n").await;
    commit(dir.path(), &["a.txt".to_owned()], "start")
        .await
        .unwrap();
    write(dir.path(), "a.txt", "two\n").await;
    write(dir.path(), "before.txt", "earlier\n").await;

    let start = snapshot(dir.path(), store.path()).await.unwrap();
    write(dir.path(), "a.txt", "three\n").await;
    write(dir.path(), "new.txt", "added\n").await;
    let diff = turn(dir.path(), store.path(), &start).await.unwrap();

    let files: Vec<_> = diff
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.change))
        .collect();
    assert_eq!(
        files,
        [("a.txt", Change::Modified), ("new.txt", Change::Added)]
    );
    let patch = diff.files[0].patch.as_deref().unwrap();
    assert!(patch.contains("-two") && patch.contains("+three"));
}

#[tokio::test]
async fn leaves_the_repository_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "a.txt", "one\n").await;
    commit(dir.path(), &["a.txt".to_owned()], "start")
        .await
        .unwrap();
    write(dir.path(), "new.txt", "untracked\n").await;
    let objects = || async { git(dir.path(), ["count-objects", "-v"]).await.unwrap() };
    let before = objects().await;

    snapshot(dir.path(), store.path()).await.unwrap();

    assert_eq!(objects().await, before);
    let status = git(dir.path(), ["status", "--porcelain"]).await.unwrap();
    assert_eq!(String::from_utf8_lossy(&status), "?? new.txt\n");
}

#[tokio::test]
async fn records_the_whole_repository_from_a_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    tokio::fs::create_dir(dir.path().join("src")).await.unwrap();
    write(dir.path(), "a.txt", "one\n").await;
    commit(dir.path(), &["a.txt".to_owned()], "start")
        .await
        .unwrap();
    let folder = dir.path().join("src");

    let start = snapshot(&folder, store.path()).await.unwrap();
    write(dir.path(), "a.txt", "two\n").await;
    write(&folder, "b.txt", "new\n").await;
    let diff = turn(&folder, store.path(), &start).await.unwrap();

    let paths: Vec<_> = diff.files.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(paths, ["a.txt", "src/b.txt"]);
}

#[tokio::test]
async fn runs_no_clean_filter_that_could_write_into_the_repository() {
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), ".gitattributes", "*.bin filter=lfs\n").await;
    commit(dir.path(), &[".gitattributes".to_owned()], "start")
        .await
        .unwrap();
    git(
        dir.path(),
        ["config", "filter.lfs.clean", "echo x >> .git/cleaned; cat"],
    )
    .await
    .unwrap();
    git(dir.path(), ["config", "filter.lfs.required", "true"])
        .await
        .unwrap();
    write(dir.path(), "model.bin", "weights\n").await;

    let start = snapshot(dir.path(), store.path()).await.unwrap();
    write(dir.path(), "model.bin", "tuned\n").await;
    let diff = turn(dir.path(), store.path(), &start).await.unwrap();

    assert!(!dir.path().join(".git").join("cleaned").exists());
    assert_eq!(diff.files.len(), 1);
    assert!(diff.files[0].patch.as_deref().unwrap().contains("+tuned"));
}

#[tokio::test]
async fn runs_no_hook_of_the_repository() {
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "a.txt", "one\n").await;
    commit(dir.path(), &["a.txt".to_owned()], "start")
        .await
        .unwrap();
    std::fs::create_dir_all(dir.path().join("hooks")).unwrap();
    write(
        dir.path(),
        "hooks/post-index-change",
        "#!/bin/sh\necho x >> .git/hooked\n",
    )
    .await;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(
            dir.path().join("hooks/post-index-change"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    git(dir.path(), ["config", "core.hooksPath", "hooks"])
        .await
        .unwrap();
    write(
        dir.path(),
        "a.txt",
        "staged
",
    )
    .await;
    git(dir.path(), ["add", "a.txt"]).await.unwrap();
    assert!(dir.path().join(".git").join("hooked").exists());
    std::fs::remove_file(dir.path().join(".git").join("hooked")).unwrap();

    let start = snapshot(dir.path(), store.path()).await.unwrap();
    write(dir.path(), "a.txt", "two\n").await;
    turn(dir.path(), store.path(), &start).await.unwrap();

    assert!(!dir.path().join(".git").join("hooked").exists());
}
