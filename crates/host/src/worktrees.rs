use std::path::Path;

use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::worktrees::{
    CreateWorktree, CreateWorktreeParams, RemoveWorktree, RemoveWorktreeParams,
};
use tokio::fs;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

const PREFIX: &str = "fogde/";

pub fn register(router: &mut Router) {
    router.add::<CreateWorktree>();
    router.add::<RemoveWorktree>();
}

impl Handler for CreateWorktree {
    async fn handle(session: &Session, params: CreateWorktreeParams) -> Result<String, Failure> {
        let repo = folders::absolute(&params.repo).map_err(failure)?;
        let name = name(&params.name)?;
        let project = repo
            .file_name()
            .ok_or_else(|| Failure::new(SERVER_ERROR, "choose a repository folder"))?;
        let path = session.shared.worktrees().join(project).join(name);

        git::add_worktree(&repo, &path, &format!("{PREFIX}{name}"))
            .await
            .map_err(failure)?;

        Ok(listed(&repo, &path)
            .await
            .unwrap_or_else(|| path.display().to_string()))
    }
}

async fn listed(repo: &Path, path: &Path) -> Option<String> {
    let wanted = fs::canonicalize(path).await.ok()?;

    for checkout in crate::workspaces::checkouts(repo).await {
        if fs::canonicalize(&checkout.path)
            .await
            .is_ok_and(|real| real == wanted)
        {
            return Some(checkout.path);
        }
    }

    None
}

impl Handler for RemoveWorktree {
    async fn handle(session: &Session, params: RemoveWorktreeParams) -> Result<(), Failure> {
        let repo = folders::absolute(&params.repo).map_err(failure)?;
        let path = folders::absolute(&params.path).map_err(failure)?;

        let branch = git::remove_worktree(&repo, &path).await.map_err(failure)?;
        if let Err(error) = crate::workspaces::close_notes_in(session, &path).await {
            tracing::warn!(error = %error.message, "could not close the notes in a removed worktree");
        }
        let Some(branch) = branch.filter(|branch| branch.starts_with(PREFIX)) else {
            return Ok(());
        };
        match git::remove_merged_branch(&repo, &branch).await {
            Ok(true) => {}
            Ok(false) => tracing::info!(%branch, "kept a branch with unmerged commits"),
            Err(error) => tracing::warn!(%error, %branch, "could not delete a merged branch"),
        }

        Ok(())
    }
}

fn name(name: &str) -> Result<&str, Failure> {
    let name = name.trim();
    let valid = !name.is_empty()
        && name.len() <= 64
        && !name.starts_with(['.', '-'])
        && !name.to_ascii_lowercase().ends_with(".lock")
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if !valid {
        return Err(Failure::new(
            SERVER_ERROR,
            "use letters, digits, dashes, dots and underscores",
        ));
    }

    Ok(name)
}

#[cfg(test)]
mod tests {
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

    #[tokio::test]
    async fn a_new_worktree_takes_the_path_git_lists() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("app");
        fs::create_dir(&repo).await.unwrap();
        run(&repo, &["init", "--quiet", "--initial-branch", "main"]).await;
        run(
            &repo,
            &["commit", "--quiet", "--allow-empty", "-m", "first"],
        )
        .await;
        let path = dir.path().join("login");
        git::add_worktree(&repo, &path, "feature/login")
            .await
            .unwrap();

        let listed = listed(&repo, &path).await;

        assert!(listed.is_some());
        assert_eq!(
            listed,
            crate::workspaces::checkouts(&repo)
                .await
                .into_iter()
                .map(|checkout| checkout.path)
                .find(|checkout| !checkout.ends_with("app"))
        );
    }

    #[test]
    fn accepts_plain_branch_names_only() {
        assert_eq!(name(" fix-login_2.0 ").ok(), Some("fix-login_2.0"));

        for bad in [
            "", "..", "../etc", "a/b", r"a\b", "-x", ".x", "x.lock", "a b",
        ] {
            assert!(name(bad).is_err(), "{bad}");
        }
    }
}
