use github::SignIn;
use protocol::github::{Account, FinishSignIn, GetAccounts, SignInCode, StartSignIn};
use protocol::rpc::{Failure, SERVER_ERROR};
use serde_json::Value;

use super::{Session, cancelled, failure, parse, reply};

impl Session {
    pub(super) async fn accounts(&self, params: Value) -> Result<Value, Failure> {
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

    pub(super) async fn start_sign_in(&self, params: Value) -> Result<Value, Failure> {
        let host = github::host(&parse::<StartSignIn>(params)?).map_err(|error| failure(&error))?;
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

    pub(super) async fn finish_sign_in(&self, params: Value) -> Result<Value, Failure> {
        parse::<FinishSignIn>(params)?;
        let Some(signin) = self.signin.lock().await.take() else {
            return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
        };

        tokio::select! {
            result = signin.finish() => reply::<FinishSignIn, _>(result),
            () = self.closed() => Err(cancelled()),
        }
    }
}
