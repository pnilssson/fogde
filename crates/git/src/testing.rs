use std::path::Path;

use tokio::fs;

use crate::run::git;

pub(crate) async fn repo(path: &Path) {
    git(path, ["init", "--quiet", "--initial-branch", "main"])
        .await
        .unwrap();
    git(path, ["config", "user.name", "Test"]).await.unwrap();
    git(path, ["config", "user.email", "test@example.com"])
        .await
        .unwrap();
}

pub(crate) async fn write(root: &Path, name: &str, text: &str) {
    fs::write(root.join(name), text).await.unwrap();
}

pub(crate) async fn log(root: &Path) -> Vec<String> {
    let output = git(root, ["log", "--format=%s"]).await.unwrap();

    String::from_utf8_lossy(&output)
        .lines()
        .map(str::to_owned)
        .collect()
}
