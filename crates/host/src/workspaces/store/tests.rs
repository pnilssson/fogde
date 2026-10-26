use super::*;
use crate::db;

fn terminal(id: &str, folder: &str) -> Terminal {
    Terminal {
        id: id.to_owned(),
        name: format!("Terminal {id}"),
        agent: None,
        status: protocol::terminals::Status::Idle,
        folder: folder.to_owned(),
        branch: None,
        changes: None,
        checkout: None,
    }
}

fn ids(workspace: &Workspace) -> Vec<&str> {
    workspace
        .terminals
        .iter()
        .map(|terminal| terminal.id.as_str())
        .collect()
}

async fn placed(pool: &SqlitePool, workspace: &Workspace, terminals: &[&str]) {
    for id in terminals {
        place(pool, id, &workspace.id, None).await.unwrap();
    }
}

#[tokio::test]
async fn starts_without_workspaces() {
    let pool = db::memory().await;

    let listing = list(&pool, &[]).await.unwrap();

    assert!(listing.workspaces.is_empty());
    assert_eq!(listing.loose, Vec::<String>::new());
}

#[tokio::test]
async fn terminals_without_a_workspace_are_loose() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1"]).await;

    let listing = list(
        &pool,
        &[
            terminal("w1", "/repos/app"),
            terminal("w2", "/repos/app/src"),
            terminal("w3", ""),
        ],
    )
    .await
    .unwrap();

    assert_eq!(listing.workspaces.len(), 1);
    assert_eq!(ids(&listing.workspaces[0]), ["w1"]);
    assert_eq!(listing.loose, ["w2", "w3"]);
}

#[tokio::test]
async fn creating_a_workspace_for_a_known_folder_returns_the_existing_one() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", Some("1")).await.unwrap();

    let again = create(&pool, "/repos/app/", None).await.unwrap();

    assert_eq!(again.id, app.id);
    assert_eq!(again.identity.as_deref(), Some("1"));
    assert_eq!(list(&pool, &[]).await.unwrap().workspaces.len(), 1);
}

#[tokio::test]
async fn terminals_keep_the_checkout_they_were_placed_in() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    place(&pool, "w1", &app.id, None).await.unwrap();
    place(&pool, "w2", &app.id, Some("/worktrees/app/login"))
        .await
        .unwrap();

    let listing = list(
        &pool,
        &[
            terminal("w1", "/worktrees/app/login"),
            terminal("w2", "/repos/app"),
        ],
    )
    .await
    .unwrap();
    let checkouts: Vec<_> = listing.workspaces[0]
        .terminals
        .iter()
        .map(|terminal| terminal.checkout.as_deref())
        .collect();

    assert_eq!(checkouts, [None, Some("/worktrees/app/login")]);
}

#[tokio::test]
async fn placing_again_moves_a_terminal_between_checkouts() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    place(&pool, "w1", &app.id, Some("/worktrees/app/login"))
        .await
        .unwrap();

    place(&pool, "w1", &app.id, None).await.unwrap();
    let listing = list(&pool, &[terminal("w1", "/repos/app")]).await.unwrap();

    assert_eq!(listing.workspaces[0].terminals[0].checkout, None);
}

#[tokio::test]
async fn relocating_follows_a_renamed_folder() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();

    relocate(&pool, &app.id, "/repos/app-renamed", Some("7"))
        .await
        .unwrap();
    let workspace = &list(&pool, &[]).await.unwrap().workspaces[0];

    assert_eq!(workspace.folder, "/repos/app-renamed");
    assert_eq!(workspace.identity.as_deref(), Some("7"));
}

#[tokio::test]
async fn closed_terminals_leave_their_workspace() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2"]).await;

    let listing = list(&pool, &[terminal("w2", "/repos/app")]).await.unwrap();

    assert_eq!(ids(&listing.workspaces[0]), ["w2"]);
    assert_eq!(listing.workspaces[0].terminals[0].name, "Terminal w2");
    assert_eq!(members(&pool, &app.id).await.unwrap(), ["w2"]);
}

#[tokio::test]
async fn deleting_a_workspace_removes_it_after_listing_its_terminals() {
    let pool = db::memory().await;
    let app = create(&pool, "/repos/app", None).await.unwrap();
    placed(&pool, &app, &["w1", "w2"]).await;

    assert_eq!(members(&pool, &app.id).await.unwrap(), ["w1", "w2"]);
    delete(&pool, &app.id).await.unwrap();

    assert_eq!(members(&pool, &app.id).await.unwrap(), Vec::<String>::new());
    assert!(list(&pool, &[]).await.unwrap().workspaces.is_empty());
}

#[tokio::test]
async fn rejects_unknown_workspaces() {
    let pool = db::memory().await;

    assert!(matches!(delete(&pool, "9").await, Err(Error::Missing)));
    assert!(matches!(delete(&pool, "home").await, Err(Error::Missing)));
    assert!(matches!(members(&pool, "home").await, Err(Error::Missing)));
    assert!(matches!(
        relocate(&pool, "9", "/repos/app", None).await,
        Err(Error::Missing)
    ));
    assert!(matches!(
        place(&pool, "w1", "9", None).await,
        Err(Error::Missing)
    ));
    assert!(matches!(
        place(&pool, "w1", "home", None).await,
        Err(Error::Missing)
    ));
}

async fn order(pool: &SqlitePool, checkout: Option<&str>, terminals: &[&str]) -> Vec<String> {
    let running: Vec<_> = terminals
        .iter()
        .map(|id| terminal(id, "/repos/app"))
        .collect();

    list(pool, &running).await.unwrap().workspaces[0]
        .terminals
        .iter()
        .filter(|terminal| terminal.checkout.as_deref() == checkout)
        .map(|terminal| terminal.id.clone())
        .collect()
}

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
async fn rejects_moving_a_terminal_outside_a_workspace() {
    let pool = db::memory().await;

    assert!(matches!(
        arrange(&pool, "w1", None).await,
        Err(Error::Unplaced)
    ));
}
