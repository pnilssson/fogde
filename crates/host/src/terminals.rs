mod changes;
mod herdr;
mod images;
mod launch;
mod stream;
mod title;
mod tracker;

use std::collections::HashMap;
use std::future;
use std::sync::{Mutex, MutexGuard, PoisonError};

use futures_util::future::join_all;
use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::terminals::{
    CloseTerminal, CreateTerminal, CreateTerminalParams, PasteImage, PasteImageParams,
    ResizeTerminal, ScrollTerminal, ScrollTerminalParams, SeekTerminal, Status, Terminal,
    TerminalInput, TerminalInputParams, TerminalKey, TerminalKeyParams, UnwatchTerminal,
    ViewTerminal, WatchTerminal, WatchTerminalParams,
};
use tokio::sync::mpsc;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
pub use changes::count;
pub use launch::path_with;
use launch::{Launch, Launches, label};
use stream::Control;
pub use tracker::Tracker;

#[derive(Default)]
pub struct State {
    controls: Mutex<HashMap<String, mpsc::UnboundedSender<Control>>>,
    launches: Launches,
}

impl State {
    pub fn clear(&self) {
        self.lock().clear();
        self.launches.clear();
    }

    fn forget(&self, id: &str) {
        self.lock().remove(id);
        self.launches.take(id);
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
        self.controls.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn register(router: &mut Router) {
    router.add::<CreateTerminal>();
    router.add::<CloseTerminal>();
    router.add::<WatchTerminal>();
    router.add::<UnwatchTerminal>();
    router.add::<TerminalInput>();
    router.add::<TerminalKey>();
    router.add::<ResizeTerminal>();
    router.add::<ScrollTerminal>();
    router.add::<SeekTerminal>();
    router.add::<PasteImage>();
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
                folder: state.folder,
                branch: state.branch,
                changes: state.changes,
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
        session.terminals.forget(id);
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
        if params.agent.is_some_and(|agent| agent.spec().github) {
            env.insert("PATH".to_owned(), path_with(session.gh().await?.program()));
        }

        let checkouts = crate::workspaces::checkouts(&folder).await;
        let checkout = crate::workspaces::worktree_of(&folder, &checkouts);

        let placing = session.shared.placing.lock().await;
        let pane = herdr
            .client()
            .create_workspace(&folder, name, &env)
            .await
            .map_err(failure)?;
        let workspace = pane.workspace.clone();
        crate::workspaces::place(session, &workspace, &params.workspace, checkout.as_deref())
            .await?;
        drop(placing);
        if let Some(agent) = params.agent {
            session
                .terminals
                .launches
                .hold(workspace.clone(), Launch { pane, agent });
        }

        Ok(Terminal {
            id: workspace,
            name: name.to_owned(),
            agent: params.agent,
            status: Status::Idle,
            folder: folder.display().to_string(),
            branch: git::branch(&folder).await,
            changes: None,
            checkout,
        })
    }
}

impl Handler for CloseTerminal {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        session.terminals.forget(&id);

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
        stream::open(session, params).await
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

impl Handler for TerminalKey {
    const ORDERED: bool = true;

    fn handle(
        session: &Session,
        params: TerminalKeyParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(
            session
                .terminals
                .send(&params.id, Control::Key(herdr::key(params.key))),
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

impl Handler for PasteImage {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: PasteImageParams) -> Result<(), Failure> {
        let path = images::save(&session.shared.images(), params.image, params.data)
            .await
            .map_err(failure)?;
        let paste = images::paste(&path);

        session
            .terminals
            .send(&params.id, Control::Input(paste.into_bytes()))
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
