mod launch;
mod stream;
mod tracker;

use std::collections::HashMap;
use std::future;
use std::sync::{Mutex, MutexGuard, PoisonError};

use futures_util::future::join_all;
use protocol::agents::Agent;
use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::terminals::{
    CloseTerminal, CreateTerminal, CreateTerminalParams, ResizeTerminal, ScrollTerminal,
    ScrollTerminalParams, Status, Terminal, TerminalInput, TerminalInputParams, UnwatchTerminal,
    ViewTerminal, WatchTerminal, WatchTerminalParams,
};
use tokio::sync::mpsc;

use crate::folders;
use crate::rpc::{Handler, Router, cancelled, failure};
use crate::session::Session;
use launch::{label, path_with, start};
use stream::{Control, watch};
pub use tracker::Tracker;

#[derive(Default)]
pub struct State(Mutex<HashMap<String, mpsc::UnboundedSender<Control>>>);

impl State {
    pub fn clear(&self) {
        self.lock().clear();
    }

    fn send(&self, id: &str, control: Control) -> Result<(), Failure> {
        let sent = self
            .lock()
            .get(id)
            .is_some_and(|controls| controls.send(control).is_ok());
        if !sent {
            return Err(Failure::new(SERVER_ERROR, "the terminal is not open"));
        }

        Ok(())
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, mpsc::UnboundedSender<Control>>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn register(router: &mut Router) {
    router.add::<CreateTerminal>();
    router.add::<CloseTerminal>();
    router.add::<WatchTerminal>();
    router.add::<UnwatchTerminal>();
    router.add::<TerminalInput>();
    router.add::<ResizeTerminal>();
    router.add::<ScrollTerminal>();
    router.add::<ViewTerminal>();
}

pub async fn list(session: &Session) -> Result<Vec<Terminal>, Failure> {
    let client = session.herdr().await?.client();
    let (workspaces, panes) =
        tokio::try_join!(client.workspaces(), client.panes()).map_err(failure)?;

    Ok(workspaces
        .iter()
        .map(|workspace| {
            let pane = panes.iter().find(|pane| pane.workspace == workspace.id);
            let state = session.shared.tracker.state(workspace, pane);

            Terminal {
                id: state.id,
                name: state.name,
                agent: state.agent,
                status: state.status,
                folder: pane
                    .and_then(|pane| pane.cwd.as_ref())
                    .map(|cwd| cwd.display().to_string())
                    .unwrap_or_default(),
                checkout: None,
            }
        })
        .collect())
}

pub async fn close(session: &Session, ids: &[String]) -> Result<(), Failure> {
    if ids.is_empty() {
        return Ok(());
    }

    for id in ids {
        session.terminals.lock().remove(id);
    }
    let client = session.herdr().await?.client();
    let open = client.workspaces().await.map_err(failure)?;
    let closing = open
        .iter()
        .filter(|workspace| ids.contains(&workspace.id))
        .map(|workspace| client.close_workspace(&workspace.id));

    join_all(closing)
        .await
        .into_iter()
        .collect::<Result<(), _>>()
        .map_err(failure)
}

impl Handler for CreateTerminal {
    async fn handle(session: &Session, params: CreateTerminalParams) -> Result<Terminal, Failure> {
        let herdr = session.herdr().await?;
        let folder = folders::absolute(&params.folder).map_err(failure)?;
        let name = label(params.agent);
        let mut env = herdr.pane_env().clone();
        if matches!(params.agent, Some(Agent::Copilot)) {
            env.insert("PATH".to_owned(), path_with(session.gh().await?.program()));
        }

        let placing = session.shared.placing.lock().await;
        let pane = herdr
            .client()
            .create_workspace(&folder, name, &env)
            .await
            .map_err(failure)?;
        crate::workspaces::place(session, &pane.workspace, &params.workspace).await?;
        drop(placing);
        if let Some(agent) = params.agent {
            start(herdr.client(), &pane, agent, session.outgoing.clone());
        }

        let checkouts = crate::workspaces::checkouts(&folder).await;

        Ok(Terminal {
            id: pane.workspace,
            name: name.to_owned(),
            agent: params.agent,
            status: Status::Idle,
            folder: folder.display().to_string(),
            checkout: crate::workspaces::checkout_of(&folder, &checkouts),
        })
    }
}

impl Handler for CloseTerminal {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        session.terminals.lock().remove(&id);

        session
            .herdr()
            .await?
            .client()
            .close_workspace(&id)
            .await
            .map_err(failure)
    }
}

impl Handler for WatchTerminal {
    async fn handle(session: &Session, params: WatchTerminalParams) -> Result<(), Failure> {
        let herdr = session.herdr().await?;
        let pane = herdr
            .client()
            .panes()
            .await
            .map_err(failure)?
            .into_iter()
            .find(|pane| pane.workspace == params.id)
            .ok_or_else(|| Failure::new(SERVER_ERROR, "the terminal is gone"))?;
        let stream = herdr_client::Terminal::open(
            herdr.program(),
            &herdr.socket(),
            &pane.id,
            params.size.cols,
            params.size.rows,
        )
        .map_err(failure)?;

        let (controls, receiver) = mpsc::unbounded_channel();
        let mut watches = session.terminals.lock();
        if session.is_closed() {
            return Err(cancelled());
        }
        watches.insert(params.id.clone(), controls);
        drop(watches);
        tokio::spawn(watch(stream, params.id, receiver, session.outgoing.clone()));

        Ok(())
    }
}

impl Handler for UnwatchTerminal {
    const ORDERED: bool = true;

    fn handle(session: &Session, id: String) -> impl Future<Output = Result<(), Failure>> + Send {
        session.terminals.lock().remove(&id);

        future::ready(Ok(()))
    }
}

impl Handler for TerminalInput {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalInputParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Input(params.data.into_bytes())),
        )
    }
}

impl Handler for ResizeTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: WatchTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Resize(params.size)),
        )
    }
}

impl Handler for ScrollTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: ScrollTerminalParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let id = params.id.clone();

        future::ready(session.terminals.send(&id, Control::Scroll(params)))
    }
}

impl Handler for ViewTerminal {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        id: Option<String>,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        session.shared.tracker.view(session.id, id);

        future::ready(Ok(()))
    }
}
