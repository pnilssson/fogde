#[cfg(test)]
mod export;

use std::collections::BTreeMap;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::LazyLock;

use protocol::rpc::{
    Failure, INVALID_PARAMS, METHOD_NOT_FOUND, Message, Notification, SERVER_ERROR,
};
use protocol::{Event, Method};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::session::Session;

pub static ROUTER: LazyLock<Router> = LazyLock::new(|| {
    let mut router = Router::default();
    crate::accessibility::register(&mut router);
    crate::agents::register(&mut router);
    crate::folders::register(&mut router);
    crate::github::register(&mut router);
    crate::health::register(&mut router);
    crate::machines::register(&mut router);
    crate::notifications::register(&mut router);
    crate::onboarding::register(&mut router);
    crate::repos::register(&mut router);
    crate::server::register(&mut router);
    crate::stats::register(&mut router);
    crate::terminals::register(&mut router);
    crate::theme::register(&mut router);
    crate::workspaces::register(&mut router);
    crate::worktrees::register(&mut router);
    router
});

pub trait Handler: Method<Params: Send, Result: Send> {
    const ORDERED: bool = false;

    fn handle(
        session: &Session,
        params: Self::Params,
    ) -> impl Future<Output = Result<Self::Result, Failure>> + Send;
}

type Reply<'a> = Pin<Box<dyn Future<Output = Result<Value, Failure>> + Send + 'a>>;

struct Route {
    call: for<'a> fn(&'a Session, Value) -> Reply<'a>,
    ordered: bool,
    #[cfg(test)]
    export: fn(&mut export::Types) -> export::Entry,
}

#[derive(Default)]
pub struct Router {
    routes: BTreeMap<&'static str, Route>,
}

impl Router {
    pub fn add<M: Handler>(&mut self) {
        let route = Route {
            call: call::<M>,
            ordered: M::ORDERED,
            #[cfg(test)]
            export: export::entry::<M>,
        };
        let replaced = self.routes.insert(M::NAME, route);
        debug_assert!(replaced.is_none(), "{} is registered twice", M::NAME);
    }

    pub fn ordered(&self, method: &str) -> bool {
        self.routes.get(method).is_some_and(|route| route.ordered)
    }

    pub async fn dispatch(
        &self,
        session: &Session,
        method: &str,
        params: Value,
    ) -> Result<Value, Failure> {
        let Some(route) = self.routes.get(method) else {
            return Err(Failure::new(
                METHOD_NOT_FOUND,
                format!("unknown method {method}"),
            ));
        };

        (route.call)(session, params).await
    }
}

fn call<M: Handler>(session: &Session, params: Value) -> Reply<'_> {
    Box::pin(async move {
        let params = serde_json::from_value(params)
            .map_err(|error| Failure::new(INVALID_PARAMS, error.to_string()))?;
        let result = M::handle(session, params).await?;

        serde_json::to_value(result).map_err(|error| Failure::new(SERVER_ERROR, error.to_string()))
    })
}

pub fn notify<E: Event>(outgoing: &mpsc::UnboundedSender<Message>, params: E::Params) {
    if let Some(message) = notification::<E>(params) {
        let _ = outgoing.send(message);
    }
}

pub fn notification<E: Event>(params: E::Params) -> Option<Message> {
    match Notification::new::<E>(params) {
        Ok(notification) => Some(Message::Notification(notification)),
        Err(error) => {
            tracing::warn!(%error, "could not write a notification");
            None
        }
    }
}

pub fn failure(error: impl Display) -> Failure {
    tracing::warn!(%error, "request failed");

    Failure::new(SERVER_ERROR, error.to_string())
}

pub fn cancelled() -> Failure {
    Failure::new(SERVER_ERROR, "the sign-in was cancelled")
}
