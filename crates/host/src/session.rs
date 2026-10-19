use std::sync::Arc;

use github::Gh;
use herdr_client::Herdr;
use protocol::rpc::{Failure, Message, Request, Response};
use tokio::sync::{mpsc, watch};

use crate::rpc::{ROUTER, failure};
use crate::shared::Shared;

pub struct Session {
    pub shared: Arc<Shared>,
    pub local: bool,
    pub outgoing: mpsc::UnboundedSender<Message>,
    pub github: crate::github::State,
    pub agents: crate::agents::State,
    pub server: crate::server::State,
    pub terminals: crate::terminals::State,
    closed: watch::Sender<bool>,
}

impl Session {
    pub fn new(shared: Arc<Shared>, local: bool, outgoing: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            shared,
            local,
            outgoing,
            github: crate::github::State::default(),
            agents: crate::agents::State::default(),
            server: crate::server::State::default(),
            terminals: crate::terminals::State::default(),
            closed: watch::Sender::new(false),
        }
    }

    pub async fn handle(&self, request: Request) -> Response {
        let outcome = ROUTER.dispatch(self, &request.method, request.params).await;

        Response::new(Some(request.id), outcome)
    }

    pub fn close(&self) {
        self.closed.send_replace(true);
        self.terminals.clear();
    }

    pub fn is_closed(&self) -> bool {
        *self.closed.borrow()
    }

    pub async fn closed(&self) {
        let _ = self.closed.subscribe().wait_for(|closed| *closed).await;
    }

    pub async fn gh(&self) -> Result<&Gh, Failure> {
        self.shared.gh().await.map_err(failure)
    }

    pub async fn herdr(&self) -> Result<&Herdr, Failure> {
        self.shared.herdr().await.map_err(failure)
    }
}
