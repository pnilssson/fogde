use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use herdr_client::{Client, Error, Pane, Subscription, Workspace};
use protocol::terminals::{Status, TerminalState};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Notify, broadcast};

use super::launch;

const RETRY: Duration = Duration::from_secs(2);

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

pub struct Tracker {
    changes: broadcast::Sender<TerminalState>,
    states: Mutex<HashMap<String, TerminalState>>,
    views: Mutex<HashMap<u64, String>>,
    viewed: Notify,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            changes: broadcast::channel(256).0,
            states: Mutex::default(),
            views: Mutex::default(),
            viewed: Notify::new(),
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
        let previous = lock(&self.states)
            .get(&workspace.id)
            .map(|state| state.status);
        let viewed = self.viewed().contains(&workspace.id);

        describe(workspace, pane, previous, viewed)
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
            }
        }
    }

    async fn sync(&self, client: &Client) -> Result<Vec<String>, Error> {
        let (workspaces, panes) = tokio::try_join!(client.workspaces(), client.panes())?;
        let (seen, clearing) = self.update(&workspaces, &panes);

        for pane in clearing {
            if let Err(error) = client.focus_pane(&pane).await {
                tracing::warn!(%error, %pane, "could not mark the agent as seen");
            }
        }

        Ok(seen)
    }

    fn update(&self, workspaces: &[Workspace], panes: &[Pane]) -> (Vec<String>, Vec<String>) {
        let viewed = self.viewed();
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
            let state = describe(
                workspace,
                Some(pane),
                before.as_ref().map(|state| state.status),
                looking,
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
        *states = next;
        seen.sort();

        (seen, clearing)
    }

    fn viewed(&self) -> HashSet<String> {
        lock(&self.views).values().cloned().collect()
    }
}

fn describe(
    workspace: &Workspace,
    pane: Option<&Pane>,
    previous: Option<Status>,
    viewed: bool,
) -> TerminalState {
    let agent = pane
        .and_then(|pane| pane.agent.as_deref())
        .and_then(launch::agent)
        .or_else(|| launch::launched(&workspace.label));
    let herdr = pane.map_or(herdr_client::Status::Unknown, |pane| pane.status);

    TerminalState {
        id: workspace.id.clone(),
        name: workspace.label.clone(),
        agent,
        status: status(herdr, previous, viewed),
    }
}

fn status(herdr: herdr_client::Status, previous: Option<Status>, viewed: bool) -> Status {
    use herdr_client::Status as Herdr;

    match herdr {
        Herdr::Working => Status::Working,
        Herdr::Blocked => Status::Blocked,
        _ if viewed => Status::Idle,
        Herdr::Done => Status::Done,
        Herdr::Idle if matches!(previous, Some(Status::Working | Status::Done)) => Status::Done,
        Herdr::Idle | Herdr::Unknown => Status::Idle,
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

#[cfg(test)]
mod tests {
    use herdr_client::Status as Herdr;

    use super::*;

    #[test]
    fn an_agent_that_finishes_unseen_is_done_until_viewed() {
        let finished = status(Herdr::Idle, Some(Status::Working), false);
        let still = status(Herdr::Idle, Some(finished), false);
        let seen = status(Herdr::Idle, Some(still), true);
        let later = status(Herdr::Idle, Some(seen), false);

        assert_eq!(
            [finished, still, seen, later],
            [Status::Done, Status::Done, Status::Idle, Status::Idle]
        );
    }

    #[test]
    fn herdr_done_clears_only_when_viewed() {
        assert_eq!(status(Herdr::Done, None, false), Status::Done);
        assert_eq!(status(Herdr::Done, Some(Status::Done), true), Status::Idle);
    }

    #[test]
    fn an_agent_waiting_for_input_stays_blocked_while_viewed() {
        assert_eq!(status(Herdr::Blocked, None, true), Status::Blocked);
        assert_eq!(status(Herdr::Working, None, true), Status::Working);
    }

    #[test]
    fn an_agent_seen_finishing_is_idle() {
        assert_eq!(
            status(Herdr::Idle, Some(Status::Working), true),
            Status::Idle
        );
        assert_eq!(
            status(Herdr::Unknown, Some(Status::Done), false),
            Status::Idle
        );
    }

    #[test]
    fn views_count_once_per_session() {
        let tracker = Tracker::default();

        tracker.view(1, Some("w1".into()));
        tracker.view(2, Some("w1".into()));
        tracker.view(1, None);

        assert_eq!(tracker.viewed(), HashSet::from(["w1".to_owned()]));
        tracker.view(2, Some("w2".into()));
        assert_eq!(tracker.viewed(), HashSet::from(["w2".to_owned()]));
    }

    #[tokio::test]
    async fn a_session_that_falls_behind_gets_every_current_state() {
        let tracker = Tracker::default();
        let mut changes = tracker.changes();
        let states: Vec<_> = (0..300)
            .map(|index| TerminalState {
                id: format!("w{index}"),
                name: "Claude".into(),
                agent: None,
                status: Status::Working,
            })
            .collect();
        for state in &states {
            let _ = tracker.changes.send(state.clone());
            lock(&tracker.states).insert(state.id.clone(), state.clone());
        }

        let mut received = tracker.next(&mut changes).await.unwrap_or_default();
        received.sort_by(|a, b| a.id.cmp(&b.id));
        let mut expected = states;
        expected.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(received, expected);
    }
}
