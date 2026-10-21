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
        place(pool, id, &workspace.id).await.unwrap();
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
    let work = create(&pool, "/repos/work").await.unwrap();
    placed(&pool, &work, &["w1"]).await;

    let listing = list(
        &pool,
        &[
            terminal("w1", "/repos/work"),
            terminal("w2", "/repos/work/src"),
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
async fn new_workspaces_are_named_after_their_folder() {
    let pool = db::memory().await;

    let work = create(&pool, "/repos/work").await.unwrap();

    assert_eq!(work.name, "work");
    assert_eq!(work.folder, "/repos/work");
}

#[tokio::test]
async fn creating_a_workspace_for_a_known_folder_returns_the_existing_one() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();
    rename(&pool, &work.id, "Work").await.unwrap();

    let again = create(&pool, "/repos/work/").await.unwrap();

    assert_eq!(again.id, work.id);
    assert_eq!(again.name, "Work");
    assert_eq!(list(&pool, &[]).await.unwrap().workspaces.len(), 1);
}

#[tokio::test]
async fn renaming_trims_the_name() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();

    rename(&pool, &work.id, " Work ").await.unwrap();

    assert_eq!(list(&pool, &[]).await.unwrap().workspaces[0].name, "Work");
}

#[tokio::test]
async fn closed_terminals_leave_their_workspace() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();
    placed(&pool, &work, &["w1", "w2"]).await;

    let listing = list(&pool, &[terminal("w2", "/repos/work")]).await.unwrap();

    assert_eq!(ids(&listing.workspaces[0]), ["w2"]);
    assert_eq!(listing.workspaces[0].terminals[0].name, "Terminal w2");
    assert_eq!(members(&pool, &work.id).await.unwrap(), ["w2"]);
}

#[tokio::test]
async fn placed_terminals_stay_wherever_their_folder_is() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();
    placed(&pool, &work, &["w1"]).await;

    let listing = list(&pool, &[terminal("w1", "/home/me")]).await.unwrap();

    assert_eq!(listing.workspaces[0].folder, "/repos/work");
    assert_eq!(ids(&listing.workspaces[0]), ["w1"]);
    assert_eq!(listing.loose, Vec::<String>::new());
}

#[tokio::test]
async fn deleting_a_workspace_removes_it_after_listing_its_terminals() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();
    placed(&pool, &work, &["w1", "w2"]).await;

    assert_eq!(members(&pool, &work.id).await.unwrap(), ["w1", "w2"]);
    delete(&pool, &work.id).await.unwrap();

    assert_eq!(
        members(&pool, &work.id).await.unwrap(),
        Vec::<String>::new()
    );
    assert!(list(&pool, &[]).await.unwrap().workspaces.is_empty());
}

#[tokio::test]
async fn rejects_blank_names_and_unknown_workspaces() {
    let pool = db::memory().await;
    let work = create(&pool, "/repos/work").await.unwrap();

    assert!(matches!(
        rename(&pool, &work.id, "  ").await,
        Err(Error::Name)
    ));
    assert!(matches!(
        rename(&pool, "9", "Work").await,
        Err(Error::Missing)
    ));
    assert!(matches!(delete(&pool, "9").await, Err(Error::Missing)));
    assert!(matches!(delete(&pool, "home").await, Err(Error::Missing)));
    assert!(matches!(members(&pool, "home").await, Err(Error::Missing)));
    assert!(matches!(place(&pool, "w1", "9").await, Err(Error::Missing)));
    assert!(matches!(
        place(&pool, "w1", "home").await,
        Err(Error::Missing)
    ));
}
