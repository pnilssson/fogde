use std::path::Path;

use super::super::fixtures::*;
use super::super::{create, list};
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
async fn notes_and_terminals_share_one_order() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1"]).await;
    let note = open_note(&pool, &app.id, None, Some("/notes/plan.md"))
        .await
        .unwrap();
    placed(&pool, &app, &["w2"]).await;

    arrange(&pool, &note, Some("w1")).await.unwrap();
    arrange(&pool, "w2", Some(&note)).await.unwrap();

    assert_eq!(
        order(&pool, None, &["w1", "w2"]).await,
        ["w2", note.as_str(), "w1"]
    );
}

#[tokio::test]
async fn notes_open_in_their_checkout() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let login = Some("/worktrees/app/login");
    open_note(&pool, &app.id, login, Some("/notes/plan.md"))
        .await
        .unwrap();

    let listing = list(&pool, &[]).await.unwrap();

    assert_eq!(checkout(&listing.workspaces[0].tiles[0]), login);
}

#[tokio::test]
async fn showing_a_note_retargets_its_tile() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let note = open_note(&pool, &app.id, None, None).await.unwrap();

    show_note(&pool, &note, Some("/notes/plan.md"))
        .await
        .unwrap();
    let listing = list(&pool, &[]).await.unwrap();

    assert_eq!(notes(&listing.workspaces[0]), [Some("/notes/plan.md")]);
}

#[tokio::test]
async fn closing_a_note_removes_only_that_tile() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1"]).await;
    let note = open_note(&pool, &app.id, None, None).await.unwrap();

    close(&pool, &note).await.unwrap();
    close(&pool, "w1").await.unwrap();

    assert_eq!(order(&pool, None, &["w1"]).await, ["w1"]);
    assert!(matches!(
        show_note(&pool, &note, None).await,
        Err(Error::Closed)
    ));
}

#[tokio::test]
async fn rejects_notes_in_unknown_workspaces() {
    let pool = db::memory().await;

    assert!(matches!(
        open_note(&pool, "9", None, None).await,
        Err(Error::Missing)
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
    open_note(&pool, &app.id, Some("/worktrees/app/login"), None)
        .await
        .unwrap();

    let found = terminals_in(&pool, Path::new("/worktrees/app/login"))
        .await
        .unwrap();

    assert_eq!(found, ["w1"]);
}

#[tokio::test]
async fn renames_move_the_notes_inside() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let at = |parts: &[&str]| {
        parts
            .iter()
            .fold(Path::new("/notes").to_path_buf(), |path, part| {
                path.join(part)
            })
    };
    for note in [
        &["app.md"][..],
        &["app"],
        &["app", "plan.md"],
        &["apps", "plan.md"],
    ] {
        let path = at(note).display().to_string();
        open_note(&pool, &app.id, None, Some(&path)).await.unwrap();
    }
    open_note(&pool, &app.id, None, None).await.unwrap();

    assert!(follow(&pool, &at(&["app"]), &at(&["web"])).await.unwrap());
    assert!(
        !follow(&pool, &at(&["docs"]), &at(&["specs"]))
            .await
            .unwrap()
    );
    let listing = list(&pool, &[]).await.unwrap();
    let expected: Vec<_> = [
        &["app.md"][..],
        &["web"],
        &["web", "plan.md"],
        &["apps", "plan.md"],
    ]
    .iter()
    .map(|note| Some(at(note).display().to_string()))
    .chain([None])
    .collect();

    assert_eq!(
        notes(&listing.workspaces[0]),
        expected.iter().map(Option::as_deref).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn opening_the_monitor_again_reuses_its_tile() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let login = Some("/worktrees/app/login");

    let first = open_monitor(&pool, &app.id, None).await.unwrap();
    let again = open_monitor(&pool, &app.id, None).await.unwrap();
    let other = open_monitor(&pool, &app.id, login).await.unwrap();
    let listing = list(&pool, &[]).await.unwrap();

    assert_eq!(first, again);
    assert_ne!(first, other);
    assert_eq!(
        ids(&listing.workspaces[0]),
        [first.as_str(), other.as_str()]
    );
}

#[tokio::test]
async fn closing_a_monitor_removes_it() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    let main = open_monitor(&pool, &app.id, None).await.unwrap();

    close(&pool, &main).await.unwrap();
    let listing = list(&pool, &[]).await.unwrap();

    assert_eq!(ids(&listing.workspaces[0]), Vec::<&str>::new());
}

#[tokio::test]
async fn rejects_monitors_in_unknown_workspaces() {
    let pool = db::memory().await;

    assert!(matches!(
        open_monitor(&pool, "9", None).await,
        Err(Error::Missing)
    ));
}
