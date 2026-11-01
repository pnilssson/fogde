use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use futures_util::future::join_all;
use herdr_client::{Connection, Error, Herdr, Terminal};
use protocol::stats::Stats;
use protocol::terminals::{Changes, Status, TerminalState};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Notify, broadcast};

use super::turns::Turns;
use super::{SEEN, owned};
use describe::{describe, folder, running};
use models::Model;

const RETRY: Duration = Duration::from_secs(2);
const RESYNC: Duration = Duration::from_millis(1500);

#[derive(Clone, Default)]
struct Git {
    branches: HashMap<String, Option<String>>,
    changes: HashMap<String, Changes>,
}

pub struct Tracker {
    changes: broadcast::Sender<TerminalState>,
    states: Mutex<HashMap<String, TerminalState>>,
    git: Mutex<Git>,
    models: Mutex<HashMap<String, Model>>,
    detected: Mutex<HashSet<String>>,
    views: Mutex<HashMap<u64, String>>,
    marked: Mutex<HashMap<String, u64>>,
    stats: Mutex<Stats>,
    viewed: Notify,
    counted: Notify,
    recount: Notify,
    turns: Arc<Turns>,
}

enum Wake {
    Changed(Result<herdr_client::Change, Error>),
    Viewed,
    Counted,
    Resync,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            changes: broadcast::channel(256).0,
            states: Mutex::default(),
            git: Mutex::default(),
            models: Mutex::default(),
            detected: Mutex::default(),
            views: Mutex::default(),
            marked: Mutex::default(),
            stats: Mutex::default(),
            viewed: Notify::new(),
            counted: Notify::new(),
            recount: Notify::new(),
            turns: Arc::default(),
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

    pub fn state(&self, terminal: &Terminal) -> TerminalState {
        let previous = lock(&self.states).get(&terminal.address).cloned();
        let detected = lock(&self.detected).contains(&terminal.address);

        describe(
            terminal,
            previous.as_ref(),
            detected,
            &lock(&self.git),
            lock(&self.models).get(&terminal.address),
        )
    }

    pub fn set_stats(&self, stats: Stats) {
        *lock(&self.stats) = stats;
    }

    pub(super) fn stats(&self) -> Stats {
        *lock(&self.stats)
    }

    pub(super) fn folders(&self) -> Vec<String> {
        lock(&self.git).branches.keys().cloned().collect()
    }

    pub(super) fn working(&self) -> HashSet<String> {
        lock(&self.states)
            .values()
            .filter(|state| state.status == Status::Working)
            .map(|state| state.folder.clone())
            .collect()
    }

    pub fn recount(&self) {
        self.recount.notify_one();
    }

    pub(super) async fn recounted(&self) {
        self.recount.notified().await;
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

    pub async fn turn(&self, terminal: &str) -> Option<Result<git::Diff, git::Error>> {
        self.turns.diff(terminal).await
    }

    pub async fn run(self: Arc<Self>, herdr: Arc<Herdr>, store: PathBuf) {
        Turns::clear(&store).await;

        loop {
            if let Err(error) = self.follow(&herdr, &store).await {
                tracing::warn!(%error, "lost Herdr's agent states");
            }

            tokio::time::sleep(RETRY).await;
        }
    }

    async fn follow(&self, herdr: &Herdr, store: &Path) -> Result<(), Error> {
        let client = herdr.client().await?;
        let mut events = client.events().await?;
        let mut resync = tokio::time::interval(RESYNC);
        resync.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        self.read_branches(events.terminals(), true).await;
        self.sync(&client, events.terminals(), store).await;

        loop {
            let wake = tokio::select! {
                changed = events.changed() => Wake::Changed(changed),
                () = self.viewed.notified() => Wake::Viewed,
                () = self.counted.notified() => Wake::Counted,
                _ = resync.tick(), if self.watched() => Wake::Resync,
            };

            match wake {
                Wake::Changed(changed) => {
                    changed?;
                    self.read_branches(events.terminals(), false).await;
                }
                Wake::Resync => self.read_branches(events.terminals(), true).await,
                Wake::Viewed | Wake::Counted => {}
            }
            self.sync(&client, events.terminals(), store).await;
        }
    }

    async fn sync(
        &self,
        client: &Arc<Connection>,
        terminals: &BTreeMap<String, Terminal>,
        store: &Path,
    ) {
        let terminals: BTreeMap<String, Terminal> = terminals
            .iter()
            .filter(|(_, terminal)| owned(terminal))
            .map(|(address, terminal)| (address.clone(), terminal.clone()))
            .collect();
        let started = self.update(&terminals);

        self.turns
            .keep(&lock(&self.states).keys().collect::<Vec<_>>());
        self.turns.begin(store, started);
        self.mark(client, &terminals);

        if self.read_models(&terminals).await {
            self.update(&terminals);
        }
    }

    async fn read_branches(&self, terminals: &BTreeMap<String, Terminal>, always: bool) {
        let folders: HashSet<String> = terminals.values().map(folder).collect();
        if !always
            && folders
                .iter()
                .all(|folder| lock(&self.git).branches.contains_key(folder))
        {
            return;
        }

        let read = folders.into_iter().map(|folder| async move {
            let branch = git::branch(Path::new(&folder)).await;
            (folder, branch)
        });
        let branches = join_all(read).await.into_iter().collect();

        lock(&self.git).branches = branches;
    }

    fn update(&self, terminals: &BTreeMap<String, Terminal>) -> Vec<(String, PathBuf)> {
        let git = lock(&self.git).clone();
        let models = lock(&self.models).clone();
        let mut detected = lock(&self.detected);
        let mut states = lock(&self.states);
        let mut next = HashMap::new();
        let mut started = Vec::new();

        for terminal in terminals.values() {
            let before = states.remove(&terminal.address);
            if running(terminal).is_some() {
                detected.insert(terminal.address.clone());
            }
            let state = describe(
                terminal,
                before.as_ref(),
                detected.contains(&terminal.address),
                &git,
                models.get(&terminal.address),
            );
            if Turns::begins(before.as_ref(), &state) {
                started.push((state.id.clone(), PathBuf::from(&state.folder)));
            }

            if before.as_ref() != Some(&state) {
                let _ = self.changes.send(state.clone());
            }
            next.insert(state.id.clone(), state);
        }
        detected.retain(|id| next.contains_key(id));
        *states = next;

        started
    }

    pub(super) fn watched(&self) -> bool {
        !lock(&self.views).is_empty()
    }

    fn viewed(&self) -> HashSet<String> {
        lock(&self.views).values().cloned().collect()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

mod describe;
mod models;
mod seen;
#[cfg(test)]
mod tests;
