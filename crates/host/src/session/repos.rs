use futures_util::future::join_all;
use protocol::folders::ListFolders;
use protocol::repos::{
    CheckReposFolder, CloneRepo, GetReposFolder, ListRepos, RemoteRepo, SetReposFolder,
};
use protocol::rpc::Failure;
use serde_json::Value;

use super::{Session, failure, parse, reply};
use crate::folders;

impl Session {
    pub(super) async fn repos_folder(&self, params: Value) -> Result<Value, Failure> {
        parse::<GetReposFolder>(params)?;

        reply::<GetReposFolder, _>(
            self.shared
                .repos()
                .await
                .map(|path| path.display().to_string()),
        )
    }

    pub(super) async fn check_repos_folder(&self, params: Value) -> Result<Value, Failure> {
        let path = folders::absolute(&parse::<CheckReposFolder>(params)?)
            .map_err(|error| failure(&error))?;

        reply::<CheckReposFolder, _>(folders::usable(&path).await)
    }

    pub(super) async fn set_repos_folder(&self, params: Value) -> Result<Value, Failure> {
        let path = folders::absolute(&parse::<SetReposFolder>(params)?)
            .map_err(|error| failure(&error))?;

        reply::<SetReposFolder, _>(self.shared.set_repos(path).await)
    }

    pub(super) async fn folders(&self, params: Value) -> Result<Value, Failure> {
        let path =
            folders::absolute(&parse::<ListFolders>(params)?).map_err(|error| failure(&error))?;

        reply::<ListFolders, _>(folders::list(&path).await)
    }

    pub(super) async fn repos(&self, params: Value) -> Result<Value, Failure> {
        let host = github::host(&parse::<ListRepos>(params)?).map_err(|error| failure(&error))?;
        let (found, folder) = tokio::join!(
            async {
                self.gh()
                    .await?
                    .repos(&host)
                    .await
                    .map_err(|error| failure(&error))
            },
            self.shared.repos(),
        );
        let found = found?;
        let folder = folder.map_err(|error| failure(&error))?;

        let cloned = join_all(found.iter().map(|repo| repo.cloned_in(&folder))).await;
        let repos = found
            .into_iter()
            .zip(cloned)
            .map(|(repo, cloned)| RemoteRepo {
                host: repo.host,
                owner: repo.owner,
                name: repo.name,
                cloned,
            })
            .collect();

        reply::<ListRepos, github::Error>(Ok(repos))
    }

    pub(super) async fn clone_repo(&self, params: Value) -> Result<Value, Failure> {
        let repo = parse::<CloneRepo>(params)?;
        let repo = github::Repo {
            host: github::host(&repo.host).map_err(|error| failure(&error))?,
            owner: repo.owner,
            name: repo.name,
        };
        let folder = self.shared.repos().await.map_err(|error| failure(&error))?;

        reply::<CloneRepo, _>(self.gh().await?.clone_repo(&repo, &folder).await)
    }
}
