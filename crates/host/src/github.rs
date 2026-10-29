mod actions;
mod items;

use github::SignIn;
use protocol::github::{Account, CancelSignIn, FinishSignIn, GetAccounts, SignInCode, StartSignIn};
use protocol::rpc::{Failure, SERVER_ERROR};
use tokio::sync::Mutex;

use crate::rpc::{Handler, Router, cancelled, failure};
use crate::session::Session;

#[derive(Default)]
pub struct State {
    signin: Mutex<Option<SignIn>>,
}

pub fn register(router: &mut Router) {
    router.add::<GetAccounts>();
    router.add::<StartSignIn>();
    router.add::<FinishSignIn>();
    router.add::<CancelSignIn>();
    items::register(router);
    actions::register(router);
}

impl Handler for GetAccounts {
    async fn handle(session: &Session, (): ()) -> Result<Vec<Account>, Failure> {
        let accounts = session.shared.accounts().await.map_err(failure)?;

        Ok(accounts
            .into_iter()
            .map(|account| Account {
                host: account.host,
                login: account.login,
            })
            .collect())
    }
}

impl Handler for StartSignIn {
    async fn handle(session: &Session, host: String) -> Result<SignInCode, Failure> {
        let host = github::host(&host).map_err(failure)?;
        let signin = SignIn::start(session.gh().await?, host)
            .await
            .map_err(failure)?;
        let code = SignInCode {
            code: signin.code.clone(),
            url: signin.url.clone(),
        };
        *session.github.signin.lock().await = Some(signin);

        Ok(code)
    }
}

impl Handler for FinishSignIn {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        let Some(signin) = session.github.signin.lock().await.take() else {
            return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
        };

        tokio::select! {
            result = signin.finish() => result.map_err(failure),
            () = session.closed() => Err(cancelled()),
        }
    }
}

impl Handler for CancelSignIn {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        session.github.signin.lock().await.take();

        Ok(())
    }
}
