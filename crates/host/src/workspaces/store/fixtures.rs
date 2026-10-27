use protocol::terminals::Terminal;
use sqlx::SqlitePool;

use super::{Workspace, list, place};

pub fn terminal(id: &str, folder: &str) -> Terminal {
    Terminal {
        id: id.to_owned(),
        name: format!("Terminal {id}"),
        agent: None,
        model: None,
        status: protocol::terminals::Status::Idle,
        folder: folder.to_owned(),
        branch: None,
        changes: None,
        checkout: None,
    }
}

pub fn ids(workspace: &Workspace) -> Vec<&str> {
    workspace
        .terminals
        .iter()
        .map(|terminal| terminal.id.as_str())
        .collect()
}

pub async fn placed(pool: &SqlitePool, workspace: &Workspace, terminals: &[&str]) {
    for id in terminals {
        place(pool, id, &workspace.id, None).await.unwrap();
    }
}

pub async fn order(pool: &SqlitePool, checkout: Option<&str>, terminals: &[&str]) -> Vec<String> {
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
