use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use protocol::stats::Stats;
use protocol::terminals::{Base, Changes, Divergence};
use tokio::time::MissedTickBehavior;

use super::Tracker;

const EVERY: Duration = Duration::from_secs(3);
const STALE: Duration = Duration::from_secs(15);
const FETCH: Duration = Duration::from_secs(150);

struct Counted {
    stamp: Option<git::Stamp>,
    at: Instant,
    changes: Option<Changes>,
}

pub async fn count(tracker: Arc<Tracker>) {
    let mut ticks = tokio::time::interval(EVERY);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut fetched = HashMap::new();
    let mut counted = HashMap::new();
    let mut working = HashSet::new();
    let mut forced = false;

    loop {
        forced |= tokio::select! {
            _ = ticks.tick() => false,
            () = tracker.recounted() => true,
        };
        if !tracker.watched() {
            continue;
        }

        let work = Work::of(tracker.stats());
        if !work.count {
            fetched.clear();
            counted.clear();
            tracker.set_changes(HashMap::new());
            continue;
        }

        let roots = roots(&tracker.folders()).await;
        if work.fetch {
            refresh(&roots, &mut fetched).await;
        }
        let now = tracker.working();
        let busy: HashSet<&PathBuf> = roots
            .iter()
            .filter(|(folder, _)| now.contains(folder) || working.contains(folder))
            .map(|(_, root)| root)
            .collect();
        working = now;

        tracker.set_changes(measure(&roots, &busy, forced, &mut counted).await);
        forced = false;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Work {
    count: bool,
    fetch: bool,
}

impl Work {
    fn of(stats: Stats) -> Self {
        Self {
            count: stats.behind || stats.lines || stats.upstream,
            fetch: stats.behind || stats.upstream,
        }
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
    let mut seen = HashSet::new();
    for root in unique {
        let Some(common) = git::common(root).await else {
            continue;
        };
        seen.insert(common.clone());
        if fetched
            .get(&common)
            .is_some_and(|last| now.duration_since(*last) < FETCH)
        {
            continue;
        }

        fetched.insert(common, now);
        let root = root.clone();
        tokio::spawn(async move {
            if let Err(error) = git::fetch(&root).await {
                tracing::debug!(%error, "could not fetch in the background");
            }
        });
    }
    fetched.retain(|common, _| seen.contains(common));
}

async fn measure(
    roots: &[(String, PathBuf)],
    busy: &HashSet<&PathBuf>,
    forced: bool,
    counted: &mut HashMap<PathBuf, Counted>,
) -> HashMap<String, Changes> {
    let unique: HashSet<&PathBuf> = roots.iter().map(|(_, root)| root).collect();
    let now = Instant::now();
    let stamps = join_all(
        unique
            .iter()
            .map(|&root| async move { (root, git::stamp(root).await) }),
    )
    .await;
    let stale = stamps.into_iter().filter(|(root, stamp)| {
        forced || busy.contains(root) || due(counted.get(*root), stamp.as_ref(), now)
    });
    let fresh = join_all(stale.map(|(root, stamp)| async move {
        let changes = git::changes(root).await.map(convert);
        (root, stamp, changes)
    }))
    .await;

    for (root, stamp, changes) in fresh {
        counted.insert(
            root.clone(),
            Counted {
                stamp,
                at: now,
                changes,
            },
        );
    }
    counted.retain(|root, _| unique.contains(root));

    roots
        .iter()
        .filter_map(|(folder, root)| {
            let changes = counted.get(root)?.changes.clone()?;
            Some((folder.clone(), changes))
        })
        .collect()
}

fn due(counted: Option<&Counted>, stamp: Option<&git::Stamp>, now: Instant) -> bool {
    counted.is_none_or(|counted| {
        counted.stamp.as_ref() != stamp || now.duration_since(counted.at) >= STALE
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn counts_again_only_when_git_moved_or_a_while_passed() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        let before = git::stamp(dir.path()).await;
        std::fs::write(
            dir.path().join("app.txt"),
            "one
",
        )
        .unwrap();
        git(dir.path(), &["add", "app.txt"]);
        let after = git::stamp(dir.path()).await;
        let now = Instant::now();
        let counted = |at| Counted {
            stamp: before.clone(),
            at,
            changes: None,
        };

        assert!(due(None, before.as_ref(), now));
        assert!(!due(Some(&counted(now)), before.as_ref(), now));
        assert!(due(Some(&counted(now)), after.as_ref(), now));
        assert!(due(Some(&counted(now)), before.as_ref(), now + STALE));
    }

    #[test]
    fn turning_every_stat_off_stops_counting_and_fetching() {
        let off = Stats {
            behind: false,
            lines: false,
            upstream: false,
        };
        let lines = Stats { lines: true, ..off };

        assert_eq!(
            [Work::of(off), Work::of(lines), Work::of(Stats::default())],
            [
                Work {
                    count: false,
                    fetch: false
                },
                Work {
                    count: true,
                    fetch: false
                },
                Work {
                    count: true,
                    fetch: true
                },
            ]
        );
    }
}
