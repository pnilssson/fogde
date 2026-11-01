use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::process::Command;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn run(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .output()
        .await?;
    if !output.status.success() {
        return Err(format!("git {args:?} failed").into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub async fn repo(root: &Path, files: &[(&str, &str)]) -> Result<PathBuf> {
    let repo = root.join("repo");
    fs::create_dir_all(&repo).await?;
    run(&repo, &["init", "--quiet", "--initial-branch", "main"]).await?;
    fs::write(repo.join(".gitignore"), ".env\nbuild/\n").await?;
    run(&repo, &["add", ".gitignore"]).await?;
    run(&repo, &["commit", "--quiet", "-m", "first"]).await?;

    for (file, text) in files {
        let path = repo.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, text).await?;
    }

    Ok(repo)
}
