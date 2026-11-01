use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use protocol::terminals::{Status, TerminalState};

#[derive(Clone)]
struct Turn {
    folder: PathBuf,
    tree: String,
}

#[derive(Default)]
pub struct Turns {
    started: Mutex<HashMap<String, Turn>>,
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
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, "could not clear earlier turns");
        }
    }

    pub fn begin(self: &Arc<Self>, store: &Path, started: Vec<(String, PathBuf)>) {
        for (terminal, folder) in started {
            let turns = Arc::clone(self);
            let store = store.to_path_buf();
            tokio::spawn(async move { turns.start(&store, terminal, folder).await });
        }
    }

    async fn start(&self, store: &Path, terminal: String, folder: PathBuf) {
        match git::snapshot(&folder, store).await {
            Ok(tree) => {
                lock(&self.started).insert(terminal, Turn { folder, tree });
            }
            Err(error) => {
                tracing::debug!(%error, %terminal, "could not record the start of a turn");
                lock(&self.started).remove(&terminal);
            }
        }
    }

    pub async fn diff(
        &self,
        store: &Path,
        terminal: &str,
    ) -> Option<Result<git::Diff, git::Error>> {
        let turn = lock(&self.started).get(terminal).cloned()?;

        Some(git::turn(&turn.folder, store, &turn.tree).await)
    }

    pub fn keep(&self, terminals: &[&String]) {
        lock(&self.started).retain(|id, _| terminals.contains(&id));
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
