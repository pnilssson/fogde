use std::time::Duration;

use tokio::fs;

use super::*;

async fn run(repo: &Path, args: &[&str]) {
    let status = tokio::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .status()
        .await
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

async fn app(dir: &Path) -> PathBuf {
    let app = dir.join("app");
    fs::create_dir(&app).await.unwrap();
    run(&app, &["init", "--quiet", "--initial-branch", "main"]).await;
    fs::write(app.join("app.txt"), "start\n").await.unwrap();
    run(&app, &["add", "app.txt"]).await;
    run(&app, &["commit", "--quiet", "-m", "start"]).await;

    app
}

async fn kept(store: &Path, wanted: usize) -> usize {
    let mut count = 0;
    for _ in 0..100 {
        let mut entries = fs::read_dir(store).await.unwrap();
        count = 0;
        while let Some(entry) = entries.next_entry().await.unwrap() {
            if entry.file_type().await.unwrap().is_dir() {
                count += 1;
            }
        }
        if count == wanted {
            return count;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    count
}

#[tokio::test]
async fn an_earlier_start_that_finishes_last_does_not_replace_a_newer_one() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("turns");
    let app = app(dir.path()).await;
    let turns = Turns::default();
    let terminal = "t1".to_owned();

    let first = turns.claim(&terminal);
    let second = turns.claim(&terminal);
    fs::write(app.join("app.txt"), "newer\n").await.unwrap();
    turns
        .start(&store, terminal.clone(), app.clone(), second)
        .await;
    fs::write(app.join("app.txt"), "later\n").await.unwrap();
    turns
        .start(&store, terminal.clone(), app.clone(), first)
        .await;

    let diff = turns.diff(&terminal).await.unwrap().unwrap();

    assert_eq!(diff.files.len(), 1);
    let patch = diff.files[0].patch.as_deref().unwrap();
    assert!(
        patch.contains("-newer") && patch.contains("+later"),
        "{patch}"
    );
}

#[tokio::test]
async fn forgets_the_objects_of_turns_it_no_longer_keeps() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("turns");
    let app = app(dir.path()).await;
    let turns = Turns::default();
    let terminal = "t1".to_owned();

    for text in ["one\n", "two\n", "three\n"] {
        fs::write(app.join("app.txt"), text).await.unwrap();
        let number = turns.claim(&terminal);
        turns
            .start(&store, terminal.clone(), app.clone(), number)
            .await;
    }
    turns.diff(&terminal).await.unwrap().unwrap();

    assert_eq!(kept(&store, 1).await, 1);

    turns.keep(&[]);

    assert_eq!(kept(&store, 0).await, 0);
    assert!(turns.diff(&terminal).await.is_none());
}
