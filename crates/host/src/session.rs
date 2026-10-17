use std::sync::Arc;

use github::{Gh, SignIn};
use protocol::Method;
use protocol::github::{FinishSignIn, GetUser, SignInCode, StartSignIn, User};
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
            GetUser::NAME => {
                parse::<GetUser>(params)?;
                reply::<GetUser>(self.gh().await?.user().await.map(|user| user.map(convert)))
            }
            StartSignIn::NAME => {
                parse::<StartSignIn>(params)?;
                let signin = SignIn::start(self.gh().await?)
                    .await
                    .map_err(|error| failure(&error))?;
                let code = SignInCode {
                    code: signin.code.clone(),
                    url: signin.url.clone(),
                };
                *self.signin.lock().await = Some(signin);

                reply::<StartSignIn>(Ok(code))
            }
            FinishSignIn::NAME => {
                parse::<FinishSignIn>(params)?;
                let Some(signin) = self.signin.lock().await.take() else {
                    return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
                };

                reply::<FinishSignIn>(signin.finish().await.map(convert))
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

fn reply<M: Method>(result: Result<M::Result, github::Error>) -> Result<Value, Failure> {
    let result = result.map_err(|error| failure(&error))?;

    serde_json::to_value(result).map_err(|error| Failure::new(SERVER_ERROR, error.to_string()))
}

fn failure(error: &github::Error) -> Failure {
    tracing::warn!(%error, "request failed");

    Failure::new(SERVER_ERROR, error.to_string())
}

fn convert(user: github::User) -> User {
    User {
        login: user.login,
        name: user.name,
        avatar_url: user.avatar_url,
    }
}
