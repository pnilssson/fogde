mod changes;
pub mod codex;
mod files;
mod input;
mod launch;
mod stream;
mod title;
mod tracker;
mod turns;

use std::collections::{BTreeMap, HashMap};
use std::future;
use std::sync::{Mutex, MutexGuard, PoisonError};

use futures_util::future::join_all;
use herdr_client::{Run, Size};
use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::terminals::{
    CloseTerminal, CreateTerminal, CreateTerminalParams, DisplayTerminal, FocusTerminal, PasteFile,
    ResizeTerminal, Status, Terminal, TerminalInput, TerminalReply, UnwatchTerminal, ViewTerminal,
    WatchTerminal, WatchTerminalParams,
};
use tokio::sync::mpsc;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
pub use changes::count;
pub use files::prune;
use launch::Launches;
use stream::Control;
pub use tracker::Tracker;

const OWNER: &str = "fogde";
const SEEN: &str = "herdr.seen";

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
    router.add::<TerminalReply>();
    router.add::<FocusTerminal>();
    router.add::<ResizeTerminal>();
    router.add::<DisplayTerminal>();
    router.add::<PasteFile>();
    router.add::<ViewTerminal>();
}

pub fn owned(terminal: &herdr_client::Terminal) -> bool {
    terminal.labels.contains_key(OWNER)
}

pub async fn list(session: &Session) -> Result<Vec<Terminal>, Failure> {
    let client = session.herdr().await?.client().await.map_err(failure)?;
    let terminals = client.terminals().await.map_err(failure)?;

    Ok(terminals
        .iter()
        .filter(|terminal| owned(terminal))
        .map(|terminal| {
            let state = session.shared.tracker.state(terminal);

            Terminal {
                id: state.id,
                name: state.name,
                agent: state.agent,
                model: state.model,
                status: state.status,
                active: state.active,
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
    let client = session.herdr().await?.client().await.map_err(failure)?;

    join_all(ids.iter().map(|id| client.kill(id)))
        .await
        .into_iter()
        .collect::<Result<(), _>>()
        .map_err(failure)
}

impl Handler for CreateTerminal {
    async fn handle(session: &Session, params: CreateTerminalParams) -> Result<Terminal, Failure> {
        let herdr = session.herdr().await?;
        let folder = folders::absolute(&params.folder).map_err(failure)?;
        let shell =
            crate::shell::program(session.shared.settings.shell().await.map_err(failure)?).await;
        let name = match params.agent {
            Some(agent) => agent.spec().short.to_owned(),
            None => crate::shell::name(&shell),
        };
        let gh = if params.agent.is_some_and(|agent| agent.spec().github) {
            Some(session.gh().await?.program().to_path_buf())
        } else {
            None
        };
        let env = session.shared.codex().await.env(gh.as_deref());
        let labels = BTreeMap::from([
            (OWNER.to_owned(), name.clone()),
            (SEEN.to_owned(), "0".to_owned()),
        ]);

        let checkouts = crate::workspaces::checkouts(&folder).await;
        let checkout = crate::workspaces::worktree_of(&folder, &checkouts);

        let client = herdr.client().await.map_err(failure)?;
        let placing = session.shared.placing.lock().await;
        let started = client
            .run(Run {
                argv: &[shell],
                cwd: &folder,
                env: &env,
                labels: &labels,
                size: params.size.map(|size| Size {
                    cols: size.cols,
                    rows: size.rows,
                }),
            })
            .await
            .map_err(failure)?;
        let id = started.address;
        crate::workspaces::place(session, &id, &params.workspace, checkout.as_deref()).await?;
        drop(placing);
        if let Some(agent) = params.agent {
            session.terminals.launches.hold(id.clone(), agent);
        }

        Ok(Terminal {
            id,
            name,
            agent: params.agent,
            model: None,
            status: Status::Idle,
            active: None,
            folder: folder.display().to_string(),
            branch: git::branch(&folder).await,
            changes: None,
            checkout,
        })
    }
}

impl Handler for CloseTerminal {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        close(session, &[id]).await
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
