use std::path::Path;
use std::sync::{Arc, Once};
use std::time::Duration;

use futures_util::future::join_all;
use tokio::sync::broadcast;
use tokio::time::MissedTickBehavior;

use super::store;
use crate::shared::Shared;

const EVERY: Duration = Duration::from_millis(1500);

type Heads = Vec<Vec<(String, String)>>;

pub struct Watch {
    changes: broadcast::Sender<()>,
    started: Once,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            changes: broadcast::channel(16).0,
            started: Once::new(),
        }
    }
}

pub fn changes(shared: &Arc<Shared>) -> broadcast::Receiver<()> {
    shared.watch.started.call_once(|| {
        tokio::spawn(run(Arc::clone(shared)));
    });

    shared.watch.changes.subscribe()
}

async fn run(shared: Arc<Shared>) {
    let mut ticks = tokio::time::interval(EVERY);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut seen = None;

    loop {
        ticks.tick().await;
        if shared.watch.changes.receiver_count() == 0 {
            seen = None;
            continue;
        }

        let Some(current) = heads(&shared).await else {
            continue;
        };
        if seen.as_ref().is_some_and(|seen| *seen != current) {
            let _ = shared.watch.changes.send(());
        }
        seen = Some(current);
    }
}

async fn heads(shared: &Shared) -> Option<Heads> {
    let pool = shared.db.pool().await.ok()?;
    let folders = match store::folders(pool).await {
        Ok(folders) => folders,
        Err(error) => {
            tracing::warn!(%error, "could not read the workspace folders");
            return None;
        }
    };

    Some(join_all(folders.iter().map(|folder| git::heads(Path::new(folder)))).await)
}
