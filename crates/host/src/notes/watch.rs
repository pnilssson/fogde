use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use tokio::sync::{Mutex, broadcast};

use crate::shared::Shared;

const SETTLE: Duration = Duration::from_millis(200);

pub struct Notes {
    changes: broadcast::Sender<()>,
    watching: Mutex<Option<Watching>>,
}

pub(super) struct Watching {
    root: PathBuf,
    _debouncer: Debouncer<RecommendedWatcher>,
}

impl Default for Notes {
    fn default() -> Self {
        Self {
            changes: broadcast::channel(16).0,
            watching: Mutex::default(),
        }
    }
}

pub fn changes(shared: &Shared) -> broadcast::Receiver<()> {
    shared.notes.changes.subscribe()
}

pub(super) fn changed(shared: &Shared) {
    let _ = shared.notes.changes.send(());
}

pub(super) async fn watch(shared: &Shared, root: &Path) {
    let mut watching = shared.notes.watching.lock().await;
    if watching
        .as_ref()
        .is_some_and(|current| current.root == root)
    {
        return;
    }

    *watching = None;
    let changes = shared.notes.changes.clone();
    let root = root.to_path_buf();
    match tokio::task::spawn_blocking(move || start(root, changes)).await {
        Ok(Ok(started)) => *watching = Some(started),
        Ok(Err(error)) => tracing::warn!(%error, "could not watch the notes folder"),
        Err(error) => tracing::warn!(%error, "could not watch the notes folder"),
    }
}

pub(super) fn start(
    root: PathBuf,
    changes: broadcast::Sender<()>,
) -> notify_debouncer_mini::notify::Result<Watching> {
    let roots: Vec<PathBuf> = std::fs::canonicalize(&root)
        .into_iter()
        .chain([root.clone()])
        .collect();
    let mut debouncer = new_debouncer(SETTLE, move |result: DebounceEventResult| {
        let Ok(events) = result else {
            return;
        };
        if events.iter().any(|event| visible(&event.path, &roots)) {
            let _ = changes.send(());
        }
    })?;
    debouncer.watcher().watch(&root, RecursiveMode::Recursive)?;

    Ok(Watching {
        root,
        _debouncer: debouncer,
    })
}

pub(super) fn visible(path: &Path, roots: &[PathBuf]) -> bool {
    let relative = roots
        .iter()
        .find_map(|root| path.strip_prefix(root).ok())
        .unwrap_or(path);

    !relative.components().any(
        |part| matches!(part, Component::Normal(name) if name.to_string_lossy().starts_with('.')),
    )
}
