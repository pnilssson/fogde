use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use protocol::terminals::{Base, Changes, Divergence};
use tokio::time::MissedTickBehavior;

use super::Tracker;

const EVERY: Duration = Duration::from_secs(3);
const FETCH: Duration = Duration::from_secs(150);
const FETCH_LIMIT: Duration = Duration::from_secs(60);

pub async fn count(tracker: Arc<Tracker>) {
    let mut ticks = tokio::time::interval(EVERY);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut fetched = HashMap::new();

    loop {
        ticks.tick().await;
        if !tracker.watched() {
            continue;
        }

        let roots = roots(&tracker.folders()).await;
        refresh(&roots, &mut fetched).await;
        tracker.set_changes(measure(&roots).await);
    }
}

async fn roots(folders: &[String]) -> Vec<(String, PathBuf)> {
    let found = join_all(folders.iter().map(|folder| async move {
        git::root(Path::new(folder))
            .await
            .map(|root| (folder.clone(), root))
    }))
    .await;

    found.into_iter().flatten().collect()
}

async fn refresh(roots: &[(String, PathBuf)], fetched: &mut HashMap<PathBuf, Instant>) {
    let now = Instant::now();
    let unique: HashSet<&PathBuf> = roots.iter().map(|(_, root)| root).collect();
    for root in unique {
        let Some(common) = git::common(root).await else {
            continue;
        };
        if fetched
            .get(&common)
            .is_some_and(|last| now.duration_since(*last) < FETCH)
        {
            continue;
        }

        fetched.insert(common, now);
        let root = root.clone();
        tokio::spawn(async move {
            match tokio::time::timeout(FETCH_LIMIT, git::fetch(&root)).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => tracing::debug!(%error, "could not fetch in the background"),
                Err(_) => tracing::debug!("a background fetch took too long"),
            }
        });
    }
}

async fn measure(roots: &[(String, PathBuf)]) -> HashMap<String, Changes> {
    let unique: HashSet<&PathBuf> = roots.iter().map(|(_, root)| root).collect();
    let counted: HashMap<&PathBuf, Changes> = join_all(
        unique
            .into_iter()
            .map(|root| async move { (root, git::changes(root).await) }),
    )
    .await
    .into_iter()
    .filter_map(|(root, changes)| changes.map(|changes| (root, convert(changes))))
    .collect();

    roots
        .iter()
        .filter_map(|(folder, root)| {
            counted
                .get(root)
                .map(|changes| (folder.clone(), changes.clone()))
        })
        .collect()
}

fn convert(changes: git::Changes) -> Changes {
    Changes {
        added: changes.added,
        removed: changes.removed,
        base: changes.base.map(|base| Base {
            name: base.name,
            behind: base.behind,
        }),
        upstream: changes.upstream.map(|upstream| Divergence {
            ahead: upstream.ahead,
            behind: upstream.behind,
        }),
    }
}
