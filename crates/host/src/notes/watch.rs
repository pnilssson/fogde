use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use protocol::notes::NotesChanged;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, broadcast};

use super::{list, markdown, note};
use crate::folders;
use crate::shared::Shared;

const SETTLE: Duration = Duration::from_millis(200);

pub struct Notes {
    changes: broadcast::Sender<NotesChanged>,
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

pub fn changes(shared: &Shared) -> broadcast::Receiver<NotesChanged> {
    shared.notes.changes.subscribe()
}

pub async fn next(receiver: &mut broadcast::Receiver<NotesChanged>) -> Option<NotesChanged> {
    match receiver.recv().await {
        Ok(update) => Some(update),
        Err(RecvError::Lagged(_)) => Some(everything()),
        Err(RecvError::Closed) => None,
    }
}

pub(super) fn changed(shared: &Shared) {
    let _ = shared.notes.changes.send(everything());
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
    changes: broadcast::Sender<NotesChanged>,
) -> notify_debouncer_mini::notify::Result<Watching> {
    let roots: Vec<PathBuf> = std::fs::canonicalize(&root)
        .into_iter()
        .chain([root.clone()])
        .collect();
    let mut known = known(&root);
    let base = root.clone();
    let mut debouncer = new_debouncer(SETTLE, move |result: DebounceEventResult| {
        let Ok(events) = result else {
            return;
        };
        let paths: HashSet<PathBuf> = events
            .into_iter()
            .filter(|event| visible(&event.path, &roots))
            .map(|event| rebase(&event.path, &roots, &base))
            .collect();
        let update = classify(paths, &mut known);
        if update.listing || !update.notes.is_empty() {
            let _ = changes.send(update);
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

pub(super) fn classify(paths: HashSet<PathBuf>, known: &mut HashSet<PathBuf>) -> NotesChanged {
    let mut changed = NotesChanged::default();
    for path in paths {
        let Ok(metadata) = std::fs::metadata(&path) else {
            if known.contains(&path) {
                known.retain(|entry| !entry.starts_with(&path));
                changed.listing = true;
            }
            continue;
        };
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let listed = metadata.is_dir() || (metadata.is_file() && markdown(&path));
        if !listed || folders::hidden(&name, &metadata) {
            continue;
        }

        if known.insert(path.clone()) {
            changed.listing = true;
        } else if metadata.is_file()
            && let Some(folder) = path.parent()
        {
            changed.notes.push(note(&path, folder, &metadata));
        }
    }

    changed
}

pub(super) fn known(root: &Path) -> HashSet<PathBuf> {
    let Ok(listing) = list(root) else {
        return HashSet::new();
    };

    listing
        .folders
        .into_iter()
        .chain(listing.notes.into_iter().map(|note| note.path))
        .map(PathBuf::from)
        .chain([root.to_path_buf()])
        .collect()
}

fn rebase(path: &Path, roots: &[PathBuf], base: &Path) -> PathBuf {
    roots
        .iter()
        .find_map(|root| path.strip_prefix(root).ok())
        .map_or_else(
            || path.to_path_buf(),
            |relative| {
                relative
                    .components()
                    .fold(base.to_path_buf(), |path, part| path.join(part))
            },
        )
}

fn everything() -> NotesChanged {
    NotesChanged {
        notes: Vec::new(),
        listing: true,
    }
}
