use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use protocol::terminals::{Status, TerminalState};

#[derive(Clone)]
struct Turn {
    folder: PathBuf,
    tree: String,
    store: Arc<Store>,
}

struct Store(PathBuf);

impl Drop for Store {
    fn drop(&mut self) {
        let path = std::mem::take(&mut self.0);
        match tokio::runtime::Handle::try_current() {
            Ok(runtime) => {
                runtime.spawn_blocking(move || forget(&path));
            }
            Err(_) => forget(&path),
        }
    }
}

#[derive(Default)]
pub struct Turns {
    latest: Mutex<HashMap<String, u64>>,
    started: Mutex<HashMap<String, Turn>>,
    next: AtomicU64,
}

impl Turns {
    pub fn begins(before: Option<&TerminalState>, state: &TerminalState) -> bool {
        state.status == Status::Working
            && state.agent.is_some()
            && !matches!(
                before.map(|before| before.status),
                Some(Status::Working | Status::Blocked)
            )
    }

    pub async fn clear(store: &Path) {
        if let Err(error) = tokio::fs::remove_dir_all(store).await
            && error.kind() != ErrorKind::NotFound
        {
            tracing::warn!(%error, "could not clear earlier turns");
        }
    }

    pub fn begin(self: &Arc<Self>, store: &Path, started: Vec<(String, PathBuf)>) {
        for (terminal, folder) in started {
            let turns = Arc::clone(self);
            let store = store.to_path_buf();
            let number = self.claim(&terminal);
            tokio::spawn(async move { turns.start(&store, terminal, folder, number).await });
        }
    }

    fn claim(&self, terminal: &str) -> u64 {
        let number = self.next.fetch_add(1, Ordering::Relaxed);
        lock(&self.latest).insert(terminal.to_owned(), number);

        number
    }

    async fn start(&self, store: &Path, terminal: String, folder: PathBuf, number: u64) {
        let own = Store(store.join(number.to_string()));
        let snapshot = git::snapshot(&folder, &own.0).await;

        let latest = lock(&self.latest);
        if latest.get(&terminal) != Some(&number) {
            return;
        }
        let mut started = lock(&self.started);
        match snapshot {
            Ok(tree) => {
                started.insert(
                    terminal,
                    Turn {
                        folder,
                        tree,
                        store: Arc::new(own),
                    },
                );
            }
            Err(error) => {
                tracing::debug!(%error, %terminal, "could not record the start of a turn");
                started.remove(&terminal);
            }
        }
    }

    pub async fn diff(&self, terminal: &str) -> Option<Result<git::Diff, git::Error>> {
        let turn = lock(&self.started).get(terminal).cloned()?;

        Some(git::turn(&turn.folder, &turn.store.0, &turn.tree).await)
    }

    pub fn keep(&self, terminals: &[&String]) {
        lock(&self.latest).retain(|id, _| terminals.contains(&id));
        lock(&self.started).retain(|id, _| terminals.contains(&id));
    }
}

fn forget(path: &Path) {
    if let Err(error) = std::fs::remove_dir_all(path)
        && error.kind() != ErrorKind::NotFound
    {
        tracing::debug!(%error, "could not remove the objects of a finished turn");
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
