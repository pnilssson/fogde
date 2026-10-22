use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use futures_util::future::join_all;
use herdr_client::{Client, Error, Pane, Subscription, Workspace};
use protocol::terminals::{Changes, TerminalState};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Notify, broadcast};

use describe::describe;

const RETRY: Duration = Duration::from_secs(2);
const RESYNC: Duration = Duration::from_millis(1500);

const LIFECYCLE: [Subscription; 8] = [
    Subscription::WorkspaceCreated,
    Subscription::WorkspaceClosed,
    Subscription::WorkspaceRenamed,
    Subscription::PaneCreated,
    Subscription::PaneClosed,
    Subscription::PaneExited,
    Subscription::PaneUpdated,
    Subscription::AgentDetected,
];

#[derive(Clone, Default)]
struct Git {
    branches: HashMap<String, Option<String>>,
    changes: HashMap<String, Changes>,
}

pub struct Tracker {
    changes: broadcast::Sender<TerminalState>,
    states: Mutex<HashMap<String, TerminalState>>,
    git: Mutex<Git>,
    detected: Mutex<HashSet<String>>,
    views: Mutex<HashMap<u64, String>>,
    viewed: Notify,
    counted: Notify,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            changes: broadcast::channel(256).0,
            states: Mutex::default(),
            git: Mutex::default(),
            detected: Mutex::default(),
            views: Mutex::default(),
            viewed: Notify::new(),
            counted: Notify::new(),
        }
    }
}

impl Tracker {
    pub fn changes(&self) -> broadcast::Receiver<TerminalState> {
        self.changes.subscribe()
    }

    pub async fn next(
        &self,
        changes: &mut broadcast::Receiver<TerminalState>,
    ) -> Option<Vec<TerminalState>> {
        match changes.recv().await {
            Ok(state) => Some(vec![state]),
            Err(RecvError::Lagged(_)) => {
                let states = lock(&self.states);
                *changes = self.changes.subscribe();

                Some(states.values().cloned().collect())
            }
            Err(RecvError::Closed) => None,
        }
    }

    pub fn view(&self, session: u64, terminal: Option<String>) {
        let mut views = lock(&self.views);
        let changed = match terminal {
            Some(id) => views.insert(session, id.clone()) != Some(id),
            None => views.remove(&session).is_some(),
        };
        drop(views);

        if changed {
            self.viewed.notify_one();
        }
    }

    pub fn state(&self, workspace: &Workspace, pane: Option<&Pane>) -> TerminalState {
        let previous = lock(&self.states).get(&workspace.id).cloned();
        let viewed = self.viewed().contains(&workspace.id);
        let detected = lock(&self.detected).contains(&workspace.id);

        describe(
            workspace,
            pane,
            previous.as_ref(),
            viewed,
            detected,
            &lock(&self.git),
        )
    }

    pub(super) fn folders(&self) -> Vec<String> {
        lock(&self.git).branches.keys().cloned().collect()
    }

    pub(super) fn set_changes(&self, changes: HashMap<String, Changes>) {
        let mut git = lock(&self.git);
        if git.changes == changes {
            return;
        }

        git.changes = changes;
        drop(git);
        self.counted.notify_one();
    }

    pub async fn run(self: Arc<Self>, client: Client) {
        loop {
            if let Err(error) = self.follow(&client).await {
                tracing::warn!(%error, "lost Herdr's agent states");
            }

            tokio::time::sleep(RETRY).await;
        }
    }

    async fn follow(&self, client: &Client) -> Result<(), Error> {
        let mut lifecycle = client.subscribe(&LIFECYCLE).await?;
        let mut panes = Vec::new();
        let mut statuses = client.subscribe(&[]).await?;
        let mut resync = tokio::time::interval(RESYNC);
        resync.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            let current = self.sync(client).await?;
            if current != panes {
                statuses = client.subscribe(&watching(&current)).await?;
                panes = current;
                continue;
            }

            tokio::select! {
                changed = lifecycle.next() => changed?,
                changed = statuses.next() => changed?,
                () = self.viewed.notified() => {}
                () = self.counted.notified() => {}
                _ = resync.tick(), if self.watched() => {}
            }
        }
    }

    async fn sync(&self, client: &Client) -> Result<Vec<String>, Error> {
        let (workspaces, panes) = tokio::try_join!(client.workspaces(), client.panes())?;
        self.read_branches(&panes).await;
        let (seen, clearing) = self.update(&workspaces, &panes);

        for pane in clearing {
            if let Err(error) = client.focus_pane(&pane).await {
                tracing::warn!(%error, %pane, "could not mark the agent as seen");
            }
        }

        Ok(seen)
    }

    async fn read_branches(&self, panes: &[Pane]) {
        let folders: HashSet<_> = panes.iter().filter_map(|pane| pane.cwd.clone()).collect();
        let read = folders.into_iter().map(|folder| async move {
            let branch = git::branch(&folder).await;
            (folder.display().to_string(), branch)
        });
        let branches = join_all(read).await.into_iter().collect();

        lock(&self.git).branches = branches;
    }

    fn update(&self, workspaces: &[Workspace], panes: &[Pane]) -> (Vec<String>, Vec<String>) {
        let viewed = self.viewed();
        let git = lock(&self.git).clone();
        let mut detected = lock(&self.detected);
        let mut states = lock(&self.states);
        let mut next = HashMap::new();
        let mut seen = Vec::new();
        let mut clearing = Vec::new();

        for workspace in workspaces {
            let Some(pane) = panes.iter().find(|pane| pane.workspace == workspace.id) else {
                continue;
            };
            let looking = viewed.contains(&workspace.id);
            let before = states.remove(&workspace.id);
            if pane.agent.is_some() {
                detected.insert(workspace.id.clone());
            }
            let state = describe(
                workspace,
                Some(pane),
                before.as_ref(),
                looking,
                detected.contains(&workspace.id),
                &git,
            );
            if looking && pane.status == herdr_client::Status::Done {
                clearing.push(pane.id.clone());
            }

            seen.push(pane.id.clone());
            if before.as_ref() != Some(&state) {
                let _ = self.changes.send(state.clone());
            }
            next.insert(state.id.clone(), state);
        }
        detected.retain(|id| next.contains_key(id));
        *states = next;
        seen.sort();

        (seen, clearing)
    }

    pub(super) fn watched(&self) -> bool {
        !lock(&self.views).is_empty()
    }

    fn viewed(&self) -> HashSet<String> {
        lock(&self.views).values().cloned().collect()
    }
}

fn watching(panes: &[String]) -> Vec<Subscription> {
    panes
        .iter()
        .map(|pane| Subscription::AgentStatus {
            pane_id: pane.clone(),
        })
        .collect()
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

mod describe;
#[cfg(test)]
mod tests;
