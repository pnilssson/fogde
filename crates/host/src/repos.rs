use futures_util::future::join_all;
use protocol::repos::{
    CheckReposFolder, CloneRepo, GetReposFolder, ListRepos, RemoteRepo, Repo, SetReposFolder,
};
use protocol::rpc::Failure;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetReposFolder>();
    router.add::<CheckReposFolder>();
    router.add::<SetReposFolder>();
    router.add::<ListRepos>();
    router.add::<CloneRepo>();
}

impl Handler for GetReposFolder {
    async fn handle(session: &Session, (): ()) -> Result<String, Failure> {
        let path = session.shared.repos().await.map_err(failure)?;

        Ok(path.display().to_string())
    }
}

impl Handler for CheckReposFolder {
    async fn handle(_: &Session, path: String) -> Result<(), Failure> {
        let path = folders::absolute(&path).map_err(failure)?;

        folders::usable(&path).await.map_err(failure)
    }
}

impl Handler for SetReposFolder {
    async fn handle(session: &Session, path: String) -> Result<(), Failure> {
        let path = folders::absolute(&path).map_err(failure)?;

        session.shared.set_repos(path).await.map_err(failure)
    }
}

impl Handler for ListRepos {
    async fn handle(session: &Session, host: String) -> Result<Vec<RemoteRepo>, Failure> {
        let host = github::host(&host).map_err(failure)?;
        let (found, folder) = tokio::join!(
            async { session.gh().await?.repos(&host).await.map_err(failure) },
            session.shared.repos(),
        );
        let found = found?;
        let folder = folder.map_err(failure)?;

        let cloned = join_all(found.iter().map(|repo| repo.cloned_in(&folder))).await;

        Ok(found
            .into_iter()
            .zip(cloned)
            .map(|(repo, cloned)| RemoteRepo {
                host: repo.host,
                owner: repo.owner,
                name: repo.name,
                cloned,
            })
            .collect())
    }
}

impl Handler for CloneRepo {
    async fn handle(session: &Session, repo: Repo) -> Result<(), Failure> {
        let repo = github::Repo {
            host: github::host(&repo.host).map_err(failure)?,
            owner: repo.owner,
            name: repo.name,
        };
        let folder = session.shared.repos().await.map_err(failure)?;

        session
            .gh()
            .await?
            .clone_repo(&repo, &folder)
            .await
            .map_err(failure)
    }
}
