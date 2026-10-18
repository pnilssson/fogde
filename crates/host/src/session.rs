mod agents;
mod github;
mod repos;
mod settings;

use std::convert::Infallible;
use std::fmt::Display;
use std::sync::Arc;

use ::github::{Gh, SignIn};
use protocol::Method;
use protocol::agents::{InstallAgent, ListAgents, SignInAgent};
use protocol::client::IsLocal;
use protocol::folders::ListFolders;
use protocol::github::{FinishSignIn, GetAccounts, StartSignIn};
use protocol::onboarding::{FinishOnboarding, GetOnboarded};
use protocol::repos::{CheckReposFolder, CloneRepo, GetReposFolder, ListRepos, SetReposFolder};
use protocol::rpc::{Failure, INVALID_PARAMS, METHOD_NOT_FOUND, Request, Response, SERVER_ERROR};
use protocol::theme::{GetTheme, SetTheme};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::shared::Shared;

pub struct Session {
    shared: Arc<Shared>,
    local: bool,
    signin: Mutex<Option<SignIn>>,
}

impl Session {
    pub fn new(shared: Arc<Shared>, local: bool) -> Self {
        Self {
            shared,
            local,
            signin: Mutex::new(None),
        }
    }

    async fn gh(&self) -> Result<&Gh, Failure> {
        self.shared.gh().await.map_err(|error| failure(&error))
    }

    pub async fn handle(&self, request: Request) -> Response {
        let outcome = self.dispatch(&request.method, request.params).await;

        Response::new(Some(request.id), outcome)
    }

    async fn dispatch(&self, method: &str, params: Value) -> Result<Value, Failure> {
        match method {
            IsLocal::NAME => {
                parse::<IsLocal>(params)?;
                reply::<IsLocal, Infallible>(Ok(self.local))
            }
            GetAccounts::NAME => self.accounts(params).await,
            StartSignIn::NAME => self.start_sign_in(params).await,
            FinishSignIn::NAME => self.finish_sign_in(params).await,
            ListAgents::NAME => self.agents(params).await,
            SignInAgent::NAME => self.sign_in_agent(params).await,
            InstallAgent::NAME => self.install_agent(params).await,
            GetOnboarded::NAME => self.onboarded(params).await,
            FinishOnboarding::NAME => self.finish_onboarding(params).await,
            GetReposFolder::NAME => self.repos_folder(params).await,
            CheckReposFolder::NAME => self.check_repos_folder(params).await,
            SetReposFolder::NAME => self.set_repos_folder(params).await,
            ListFolders::NAME => self.folders(params).await,
            ListRepos::NAME => self.repos(params).await,
            CloneRepo::NAME => self.clone_repo(params).await,
            GetTheme::NAME => self.theme(params).await,
            SetTheme::NAME => self.set_theme(params).await,
            _ => Err(Failure::new(
                METHOD_NOT_FOUND,
                format!("unknown method {method}"),
            )),
        }
    }
}

fn parse<M: Method>(params: Value) -> Result<M::Params, Failure> {
    serde_json::from_value(params).map_err(|error| Failure::new(INVALID_PARAMS, error.to_string()))
}

fn reply<M: Method, E: Display>(result: Result<M::Result, E>) -> Result<Value, Failure> {
    let result = result.map_err(|error| failure(&error))?;

    serde_json::to_value(result).map_err(|error| Failure::new(SERVER_ERROR, error.to_string()))
}

fn failure(error: &impl Display) -> Failure {
    tracing::warn!(%error, "request failed");

    Failure::new(SERVER_ERROR, error.to_string())
}
