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
