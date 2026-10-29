use super::*;
use crate::commit::commit;
use crate::testing::{repo, write};

fn paths(diff: &Diff) -> Vec<(&str, Change)> {
    diff.files
        .iter()
        .map(|file| (file.path.as_str(), file.change))
        .collect()
}

#[tokio::test]
async fn shows_uncommitted_changes_with_new_files_in_path_order() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "b.txt", "one\n").await;
    commit(dir.path(), &["b.txt".to_owned()], "start")
        .await
        .unwrap();
    write(dir.path(), "b.txt", "two\n").await;
    write(dir.path(), "a.txt", "new\n").await;
    write(dir.path(), "c.bin", "\0\x01").await;

    let diff = diff(dir.path(), Scope::Uncommitted).await.unwrap();

    assert_eq!(
        paths(&diff),
        [
            ("a.txt", Change::Untracked),
            ("b.txt", Change::Modified),
            ("c.bin", Change::Untracked)
        ]
    );
    assert_eq!((diff.files[0].added, diff.files[0].removed), (1, 0));
    assert_eq!((diff.files[1].added, diff.files[1].removed), (1, 1));
    assert!(diff.files[1].patch.as_deref().unwrap().contains("+two"));
    assert!(diff.files[2].binary);
    assert_eq!(diff.files[2].patch, None);
    assert!(!diff.truncated);
}

#[tokio::test]
async fn reads_a_repository_without_commits() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    write(dir.path(), "app.txt", "one\n").await;
    git(dir.path(), ["add", "app.txt"]).await.unwrap();

    let diff = diff(dir.path(), Scope::Uncommitted).await.unwrap();

    assert_eq!(paths(&diff), [("app.txt", Change::Added)]);
}

#[tokio::test]
async fn compares_the_branch_with_where_it_left_main() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    fs::create_dir(&app).await.unwrap();
    repo(&app).await;
    write(&app, "app.txt", "one\n").await;
    commit(&app, &["app.txt".to_owned()], "start")
        .await
        .unwrap();
    git(
        dir.path(),
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
    let remote = dir.path().join("remote.git");
    git(&app, ["remote", "add", "origin", remote.to_str().unwrap()])
        .await
        .unwrap();
    git(
        &app,
        ["push", "--quiet", "--set-upstream", "origin", "main"],
    )
    .await
    .unwrap();
    git(&app, ["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();
    write(&app, "login.txt", "login\n").await;
    commit(&app, &["login.txt".to_owned()], "Login")
        .await
        .unwrap();
    write(&app, "app.txt", "two\n").await;

    let branch = diff(&app, Scope::Branch).await.unwrap();
    let uncommitted = diff(&app, Scope::Uncommitted).await.unwrap();

    assert_eq!(branch.base.as_deref(), Some("main"));
    assert_eq!(
        paths(&branch),
        [("app.txt", Change::Modified), ("login.txt", Change::Added)]
    );
    assert_eq!(uncommitted.base, None);
    assert_eq!(paths(&uncommitted), [("app.txt", Change::Modified)]);
}

#[tokio::test]
async fn says_so_when_new_files_are_left_out() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    for index in 0..=UNTRACKED_FILES {
        write(dir.path(), &format!("{index:0>4}.txt"), "new\n").await;
    }

    let diff = diff(dir.path(), Scope::Uncommitted).await.unwrap();

    assert_eq!(diff.files.len(), UNTRACKED_FILES);
    assert!(diff.truncated);
}

#[test]
fn leaves_out_patches_past_the_limits() {
    let chunk = |path: &str, size: usize, added: u32| parse::Chunk {
        path: path.to_owned(),
        from: None,
        change: Change::Modified,
        added,
        removed: 0,
        binary: false,
        patch: "x".repeat(size),
    };

    let diff = limit(
        None,
        vec![
            (chunk("huge.txt", FILE_LIMIT + 1, 1), false),
            (chunk("long.txt", 10, LINE_LIMIT + 1), false),
            (chunk("big.txt", 10, 1), true),
            (chunk("fits.txt", 10, 1), false),
        ],
    );

    let large: Vec<bool> = diff.files.iter().map(|file| file.large).collect();
    assert_eq!(large, [true, true, true, false]);
    assert!(diff.files[..3].iter().all(|file| file.patch.is_none()));
    assert!(diff.files[3].patch.is_some());
    assert!(!diff.truncated);

    let many = (0..20)
        .map(|index| (chunk(&format!("{index}.txt"), FILE_LIMIT, 1), false))
        .collect();
    let cut = limit(None, many);
    assert!(cut.truncated);
    assert_eq!(
        cut.files.iter().filter(|file| file.patch.is_some()).count(),
        TOTAL_LIMIT / FILE_LIMIT
    );
}
