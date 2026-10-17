use std::fmt::Display;
use std::sync::Arc;

use github::{Gh, SignIn};
use protocol::Method;
use protocol::agents::{Agent, SignInAgent, SignedInAgents};
use protocol::github::{Account, FinishSignIn, GetAccounts, SignInCode, StartSignIn};
use protocol::onboarding::{FinishOnboarding, GetOnboarded};
use protocol::rpc::{Failure, INVALID_PARAMS, METHOD_NOT_FOUND, Request, Response, SERVER_ERROR};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::shared::Shared;

pub struct Session {
    shared: Arc<Shared>,
    signin: Mutex<Option<SignIn>>,
}

impl Session {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
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
            GetAccounts::NAME => {
                parse::<GetAccounts>(params)?;
                let accounts = self.gh().await?.accounts().await;
                reply::<GetAccounts, _>(accounts.map(|accounts| {
                    accounts
                        .into_iter()
                        .map(|account| Account {
                            host: account.host,
                            login: account.login,
                        })
                        .collect()
                }))
            }
            StartSignIn::NAME => {
                let host = github::host(&parse::<StartSignIn>(params)?)
                    .map_err(|error| failure(&error))?;
                let signin = SignIn::start(self.gh().await?, host)
                    .await
                    .map_err(|error| failure(&error))?;
                let code = SignInCode {
                    code: signin.code.clone(),
                    url: signin.url.clone(),
                };
                *self.signin.lock().await = Some(signin);

                reply::<StartSignIn, github::Error>(Ok(code))
            }
            FinishSignIn::NAME => {
                parse::<FinishSignIn>(params)?;
                let Some(signin) = self.signin.lock().await.take() else {
                    return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
                };

                reply::<FinishSignIn, _>(signin.finish().await)
            }
            SignedInAgents::NAME => {
                parse::<SignedInAgents>(params)?;
                reply::<SignedInAgents, _>(
                    agents::signed_in()
                        .await
                        .map(|agents| agents.into_iter().map(to_protocol).collect()),
                )
            }
            SignInAgent::NAME => {
                let agent = parse::<SignInAgent>(params)?;
                reply::<SignInAgent, _>(from_protocol(agent).sign_in().await)
            }
            GetOnboarded::NAME => {
                parse::<GetOnboarded>(params)?;
                reply::<GetOnboarded, _>(self.shared.settings.onboarded().await)
            }
            FinishOnboarding::NAME => {
                parse::<FinishOnboarding>(params)?;
                reply::<FinishOnboarding, _>(self.shared.settings.finish_onboarding().await)
            }
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

fn to_protocol(agent: agents::Agent) -> Agent {
    match agent {
        agents::Agent::Claude => Agent::Claude,
        agents::Agent::Codex => Agent::Codex,
    }
}

fn from_protocol(agent: Agent) -> agents::Agent {
    match agent {
        Agent::Claude => agents::Agent::Claude,
        Agent::Codex => agents::Agent::Codex,
    }
}
