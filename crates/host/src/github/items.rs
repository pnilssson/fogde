use protocol::github::{
    Detail, GetIssue, GetOptions, GetPull, ItemParams, ListIssues, ListParams, ListPulls, Listed,
    RepoOptions,
};
use protocol::repos::Repo;
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<ListPulls>();
    router.add::<ListIssues>();
    router.add::<GetPull>();
    router.add::<GetIssue>();
    router.add::<GetOptions>();
}

impl Handler for ListPulls {
    async fn handle(session: &Session, params: ListParams) -> Result<Vec<Listed>, Failure> {
        session
            .gh()
            .await?
            .pulls(&params.repo, params.filter)
            .await
            .map_err(failure)
    }
}

impl Handler for ListIssues {
    async fn handle(session: &Session, params: ListParams) -> Result<Vec<Listed>, Failure> {
        session
            .gh()
            .await?
            .issues(&params.repo, params.filter)
            .await
            .map_err(failure)
    }
}

impl Handler for GetPull {
    async fn handle(session: &Session, params: ItemParams) -> Result<Detail, Failure> {
        session
            .gh()
            .await?
            .pull(&params.repo, params.number)
            .await
            .map_err(failure)
    }
}

impl Handler for GetIssue {
    async fn handle(session: &Session, params: ItemParams) -> Result<Detail, Failure> {
        session
            .gh()
            .await?
            .issue(&params.repo, params.number)
            .await
            .map_err(failure)
    }
}

impl Handler for GetOptions {
    async fn handle(session: &Session, repo: Repo) -> Result<RepoOptions, Failure> {
        session.gh().await?.options(&repo).await.map_err(failure)
    }
}
