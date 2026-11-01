mod actions;
mod items;

use github::SignIn;
use protocol::github::{Account, CancelSignIn, FinishSignIn, GetAccounts, SignInCode, StartSignIn};
use protocol::rpc::{Failure, SERVER_ERROR};
use tokio::sync::{Mutex, oneshot};

use crate::rpc::{Handler, Router, cancelled, failure};
use crate::session::Session;

#[derive(Default)]
pub struct State {
    signin: Mutex<Flow>,
}

#[derive(Default)]
enum Flow {
    #[default]
    Idle,
    Waiting(Box<SignIn>),
    Finishing(oneshot::Sender<()>),
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
        stop(std::mem::replace(
            &mut *session.github.signin.lock().await,
            Flow::Waiting(Box::new(signin)),
        ));

        Ok(code)
    }
}

impl Handler for FinishSignIn {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        let (cancel, stopped) = oneshot::channel();
        let signin = {
            let mut flow = session.github.signin.lock().await;
            match std::mem::take(&mut *flow) {
                Flow::Waiting(signin) => {
                    *flow = Flow::Finishing(cancel);
                    signin
                }
                other => {
                    *flow = other;
                    return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
                }
            }
        };

        tokio::select! {
            result = signin.finish() => result.map_err(failure),
            _ = stopped => Err(cancelled()),
            () = session.closed() => Err(cancelled()),
        }
    }
}

impl Handler for CancelSignIn {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        stop(std::mem::take(&mut *session.github.signin.lock().await));

        Ok(())
    }
}

fn stop(flow: Flow) {
    if let Flow::Finishing(cancel) = flow {
        let _ = cancel.send(());
    }
}
