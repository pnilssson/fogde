use std::path::Path;

use git::{Base, Changes, Divergence, changes, fetch};
use tokio::fs;
use tokio::process::Command;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn run(repo: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(["-c", "init.defaultBranch=main"])
        .args(args)
        .output()
        .await?;
    if !output.status.success() {
        return Err(format!("git {args:?} failed").into());
    }

    Ok(())
}

async fn commit(repo: &Path, file: &str, text: &str) -> Result<()> {
    fs::write(repo.join(file), text).await?;
    run(repo, &["add", file]).await?;
    run(repo, &["commit", "--quiet", "-m", file]).await
}

#[tokio::test]
async fn counts_the_branch_against_main_and_its_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let origin = dir.path().join("origin.git");
    let app = dir.path().join("app");
    let other = dir.path().join("other");
    run(dir.path(), &["init", "--quiet", "--bare", "origin.git"])
        .await
        .unwrap();
    let url = origin.to_str().unwrap();
    run(dir.path(), &["clone", "--quiet", url, "app"])
        .await
        .unwrap();
    commit(&app, "readme.md", "one\ntwo\nthree\n")
        .await
        .unwrap();
    run(&app, &["push", "--quiet", "-u", "origin", "main"])
        .await
        .unwrap();

    run(dir.path(), &["clone", "--quiet", url, "other"])
        .await
        .unwrap();
    commit(&other, "news.md", "news\n").await.unwrap();
    run(&other, &["push", "--quiet"]).await.unwrap();

    run(&app, &["switch", "--quiet", "-c", "feature/login"])
        .await
        .unwrap();
    commit(&app, "readme.md", "one\nthree\nfour\nfive\n")
        .await
        .unwrap();
    run(&app, &["push", "--quiet", "-u", "origin", "feature/login"])
        .await
        .unwrap();
    commit(&app, "login.rs", "a\nb\n").await.unwrap();
    fs::write(app.join("readme.md"), "one\nthree\nfour\nfive\nsix\n")
        .await
        .unwrap();
    fs::write(app.join("notes.txt"), "x\ny\nz").await.unwrap();

    fetch(&app).await.unwrap();

    assert_eq!(
        changes(&app).await,
        Some(Changes {
            added: 8,
            removed: 1,
            base: Some(Base {
                name: "main".to_owned(),
                behind: 1,
            }),
            upstream: Some(Divergence {
                ahead: 1,
                behind: 0,
            }),
        })
    );
}

#[tokio::test]
async fn counts_uncommitted_lines_without_a_remote() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    run(dir.path(), &["init", "--quiet", "app"]).await.unwrap();
    commit(&app, "readme.md", "one\n").await.unwrap();
    fs::write(app.join("readme.md"), "two\n").await.unwrap();

    assert_eq!(
        changes(&app).await,
        Some(Changes {
            added: 1,
            removed: 1,
            base: None,
            upstream: None,
        })
    );
    assert!(fetch(&app).await.is_err());
}

#[tokio::test]
async fn counts_an_untracked_file_whose_name_starts_with_a_space() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    run(dir.path(), &["init", "--quiet", "app"]).await.unwrap();
    commit(&app, "readme.md", "one\n").await.unwrap();
    fs::write(app.join(" notes.txt"), "x\ny\n").await.unwrap();

    assert_eq!(changes(&app).await.map(|changes| changes.added), Some(2));
}

#[tokio::test]
async fn counts_from_the_last_commit_when_main_shares_no_history() {
    let dir = tempfile::tempdir().unwrap();
    let other = dir.path().join("other");
    let app = dir.path().join("app");
    run(dir.path(), &["init", "--quiet", "other"])
        .await
        .unwrap();
    commit(&other, "news.md", "news\n").await.unwrap();
    run(dir.path(), &["init", "--quiet", "app"]).await.unwrap();
    commit(&app, "readme.md", "one\n").await.unwrap();
    fs::write(app.join("readme.md"), "one\ntwo\n")
        .await
        .unwrap();
    run(&app, &["remote", "add", "origin", other.to_str().unwrap()])
        .await
        .unwrap();
    fetch(&app).await.unwrap();

    assert_eq!(
        changes(&app).await,
        Some(Changes {
            added: 1,
            removed: 0,
            base: None,
            upstream: None,
        })
    );
}
