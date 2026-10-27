use std::path::Path;

use super::super::create;
use super::super::fixtures::*;
use super::*;
use crate::db;

#[tokio::test]
async fn moving_puts_a_terminal_before_another() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2", "w3"]).await;

    arrange(&pool, "w3", Some("w1")).await.unwrap();

    assert_eq!(
        order(&pool, None, &["w1", "w2", "w3"]).await,
        ["w3", "w1", "w2"]
    );
}

#[tokio::test]
async fn moving_without_a_target_puts_a_terminal_last() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2", "w3"]).await;

    arrange(&pool, "w1", None).await.unwrap();

    assert_eq!(
        order(&pool, None, &["w1", "w2", "w3"]).await,
        ["w2", "w3", "w1"]
    );
}

#[tokio::test]
async fn moving_before_a_closed_terminal_puts_it_last() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2", "w3"]).await;

    arrange(&pool, "w1", Some("w9")).await.unwrap();

    assert_eq!(
        order(&pool, None, &["w1", "w2", "w3"]).await,
        ["w2", "w3", "w1"]
    );
}

#[tokio::test]
async fn moving_stays_within_the_checkout() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let login = Some("/worktrees/app/login");
    place(&pool, "w1", &app.id, None).await.unwrap();
    place(&pool, "w2", &app.id, login).await.unwrap();
    place(&pool, "w3", &app.id, None).await.unwrap();
    place(&pool, "w4", &app.id, login).await.unwrap();
    let all = ["w1", "w2", "w3", "w4"];

    arrange(&pool, "w3", Some("w1")).await.unwrap();
    arrange(&pool, "w4", Some("w1")).await.unwrap();

    assert_eq!(order(&pool, None, &all).await, ["w3", "w1"]);
    assert_eq!(order(&pool, login, &all).await, ["w2", "w4"]);
}

#[tokio::test]
async fn placed_terminals_join_the_end_and_keep_their_place() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2"]).await;
    arrange(&pool, "w2", Some("w1")).await.unwrap();

    placed(&pool, &app, &["w3", "w2"]).await;

    assert_eq!(
        order(&pool, None, &["w1", "w2", "w3"]).await,
        ["w2", "w1", "w3"]
    );
}

#[tokio::test]
async fn rejects_moving_a_closed_tile() {
    let pool = db::memory().await;

    assert!(matches!(
        arrange(&pool, "w1", None).await,
        Err(Error::Closed)
    ));
}

#[tokio::test]
async fn finds_only_the_terminals_in_a_worktree() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    place(&pool, "w1", &app.id, Some("/worktrees/app/login"))
        .await
        .unwrap();
    place(&pool, "w2", &app.id, Some("/worktrees/app/login-old"))
        .await
        .unwrap();
    place(&pool, "w3", &app.id, None).await.unwrap();

    let found = terminals_in(&pool, Path::new("/worktrees/app/login"))
        .await
        .unwrap();

    assert_eq!(found, ["w1"]);
}
