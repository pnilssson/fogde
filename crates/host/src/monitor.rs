mod probe;
mod sizes;

use std::collections::VecDeque;
use std::future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError};
use std::time::{Duration, Instant};

use protocol::monitor::{
    Bar, GetBar, ListSamples, ListSpaces, Sample, SamplesParams, SetBar, Space,
};
use protocol::rpc::Failure;
use tokio::task;
use tokio::time::MissedTickBehavior;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
use crate::shared::Shared;
use probe::{Probe, Shell};

const TICK: Duration = Duration::from_secs(1);
const IDLE: Duration = Duration::from_secs(10);
const KEEP: usize = 300;

pub struct Monitor {
    state: Mutex<State>,
    started: Once,
    sizes: Arc<sizes::Sizes>,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            state: Mutex::default(),
            started: Once::new(),
            sizes: Arc::default(),
        }
    }
}

#[derive(Default)]
struct State {
    samples: VecDeque<Sample>,
    read: Option<Instant>,
}

impl Monitor {
    fn read(&self, since: i64) -> Vec<Sample> {
        let mut state = self.lock();
        state.read = Some(Instant::now());

        state
            .samples
            .iter()
            .filter(|sample| sample.at > since)
            .cloned()
            .collect()
    }

    fn wanted(&self) -> bool {
        let mut state = self.lock();
        let wanted = state.read.is_some_and(|read| read.elapsed() < IDLE);
        if !wanted {
            state.samples.clear();
        }

        wanted
    }

    fn push(&self, sample: Sample) {
        let mut state = self.lock();
        if state.samples.len() == KEEP {
            state.samples.pop_front();
        }

        state.samples.push_back(sample);
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub fn register(router: &mut Router) {
    router.add::<ListSamples>();
    router.add::<ListSpaces>();
    router.add::<GetBar>();
    router.add::<SetBar>();
}

impl Handler for ListSamples {
    fn handle(
        session: &Session,
        params: SamplesParams,
    ) -> impl Future<Output = Result<Vec<Sample>, Failure>> + Send {
        let shared = &session.shared;
        shared.monitor.started.call_once(|| {
            tokio::spawn(run(Arc::clone(shared)));
        });

        future::ready(Ok(shared.monitor.read(params.since)))
    }
}

impl Handler for ListSpaces {
    async fn handle(session: &Session, paths: Vec<String>) -> Result<Vec<Space>, Failure> {
        let folders = paths
            .iter()
            .map(|path| folders::absolute(path))
            .collect::<Result<Vec<PathBuf>, _>>()
            .map_err(failure)?;
        let sizes = Arc::clone(&session.shared.monitor.sizes);
        let measured = task::spawn_blocking(move || sizes.measure(&folders))
            .await
            .map_err(failure)?;

        Ok(paths
            .into_iter()
            .zip(measured)
            .filter_map(|(path, bytes)| {
                Some(Space {
                    path,
                    bytes: bytes?,
                })
            })
            .collect())
    }
}

impl Handler for GetBar {
    async fn handle(session: &Session, (): ()) -> Result<Bar, Failure> {
        session.shared.settings.bar().await.map_err(failure)
    }
}

impl Handler for SetBar {
    async fn handle(session: &Session, bar: Bar) -> Result<(), Failure> {
        session.shared.settings.set_bar(bar).await.map_err(failure)
    }
}

async fn run(shared: Arc<Shared>) {
    let mut ticks = tokio::time::interval(TICK);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut probe = None;
    let mut repos = None;

    loop {
        ticks.tick().await;
        if !shared.monitor.wanted() {
            probe = None;
            continue;
        }

        let Some(mut current) = probe.take() else {
            probe = start().await;
            repos = shared.repos().await.ok();
            continue;
        };
        let shells = shells(&shared).await;
        let herdr = shared
            .running_herdr()
            .map(|herdr| herdr.program().to_path_buf());
        let folder = repos.clone();
        let measured = task::spawn_blocking(move || {
            let sample = current.sample(&shells, herdr.as_deref(), folder.as_deref());
            (current, sample)
        })
        .await;

        match measured {
            Ok((current, sample)) => {
                shared.monitor.push(sample);
                probe = Some(current);
            }
            Err(error) => tracing::warn!(%error, "could not sample the machine"),
        }
    }
}

async fn start() -> Option<Probe> {
    task::spawn_blocking(Probe::new)
        .await
        .inspect_err(|error| tracing::warn!(%error, "could not read the machine"))
        .ok()
}

async fn shells(shared: &Shared) -> Vec<Shell> {
    let Some(herdr) = shared.running_herdr() else {
        return Vec::new();
    };
    let listed = match herdr.client().await {
        Ok(client) => client.terminals().await,
        Err(error) => Err(error),
    };
    let terminals = match listed {
        Ok(terminals) => terminals,
        Err(error) => {
            tracing::debug!(%error, "could not list the terminals to sample");
            return Vec::new();
        }
    };

    terminals
        .into_iter()
        .filter(crate::terminals::owned)
        .filter_map(|terminal| {
            Some(Shell {
                pid: terminal.pid?,
                terminal: terminal.address,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use protocol::monitor::Memory;

    use super::*;

    fn sample(at: i64) -> Sample {
        Sample {
            at,
            cpu: 12.5,
            memory: Memory { used: 4, total: 16 },
            disk: None,
            battery: None,
            terminals: Vec::new(),
        }
    }

    #[test]
    fn keeps_the_latest_samples() {
        let monitor = Monitor::default();
        for at in 1..=310 {
            monitor.push(sample(at));
        }

        let kept = monitor.read(0);

        assert_eq!(kept.len(), KEEP);
        assert_eq!(kept.first().map(|sample| sample.at), Some(11));
        assert_eq!(monitor.read(308), [sample(309), sample(310)]);
    }

    #[test]
    fn stops_when_nobody_reads() {
        let monitor = Monitor::default();
        monitor.push(sample(1));

        assert!(!monitor.wanted());
        assert_eq!(monitor.read(0), []);
        assert!(monitor.wanted());
    }
}
