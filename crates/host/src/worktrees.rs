use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::worktrees::{
    CreateWorktree, CreateWorktreeParams, RemoveWorktree, RemoveWorktreeParams,
};

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

        Ok(path.display().to_string())
    }
}

impl Handler for RemoveWorktree {
    async fn handle(_: &Session, params: RemoveWorktreeParams) -> Result<(), Failure> {
        let repo = folders::absolute(&params.repo).map_err(failure)?;
        let path = folders::absolute(&params.path).map_err(failure)?;

        let branch = git::remove_worktree(&repo, &path).await.map_err(failure)?;
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
