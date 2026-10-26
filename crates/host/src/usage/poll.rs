use std::sync::Arc;
use std::time::{Duration, SystemTime};

use agents::limits;
use futures_util::future::join_all;
use protocol::agents::Agent;
use protocol::usage::{Shown, Span, Window};
use tokio::time::{Instant, MissedTickBehavior};

use super::{seconds, span, window};
use crate::shared::Shared;

const CHECK: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_secs(300);

pub async fn run(shared: Arc<Shared>) {
    let mut ticks = tokio::time::interval(CHECK);
    ticks.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut polled: Option<(Instant, Shown)> = None;

    loop {
        ticks.tick().await;
        if shared.usage.watching() == 0 {
            continue;
        }
        let shown = shared.settings.shown().await.unwrap_or_default();
        if polled
            .as_ref()
            .is_some_and(|(at, last)| *last == shown && at.elapsed() < POLL)
        {
            continue;
        }

        let readings = join_all(Agent::ALL.map(|agent| read(agent, &shown, &shared))).await;
        polled = Some((Instant::now(), shown));
        let at = seconds(SystemTime::now());
        for (agent, windows) in Agent::ALL.into_iter().zip(readings) {
            shared.usage.set(agent, windows, at);
        }
    }
}

async fn read(agent: Agent, shown: &Shown, shared: &Shared) -> Vec<Window> {
    if !shown.shows(agent) {
        return Vec::new();
    }

    match agent {
        Agent::Claude => limited(agent, limits::claude().await),
        Agent::Codex => limited(agent, limits::codex().await),
        Agent::Copilot => copilot(shared).await,
    }
}

fn limited(agent: Agent, limits: Result<Vec<limits::Limit>, agents::Error>) -> Vec<Window> {
    match limits {
        Ok(limits) => limits
            .into_iter()
            .map(|limit| window(span(limit.minutes), 100.0 - limit.used, limit.resets))
            .collect(),
        Err(error) => {
            tracing::debug!(%error, "could not read the {agent} limits");
            Vec::new()
        }
    }
}

async fn copilot(shared: &Shared) -> Vec<Window> {
    match agents::state(Agent::Copilot).await {
        Ok(agents::State::SignedIn) => {}
        Ok(_) => return Vec::new(),
        Err(error) => {
            tracing::debug!(%error, "could not check GitHub Copilot");
            return Vec::new();
        }
    }

    let quota = match shared.gh().await {
        Ok(gh) => gh.copilot_quota().await,
        Err(error) => Err(error),
    };
    match quota {
        Ok(quota) => quota
            .map(|quota| window(Span::Month, quota.left, quota.resets))
            .into_iter()
            .collect(),
        Err(error) => {
            tracing::debug!(%error, "could not read the GitHub Copilot quota");
            Vec::new()
        }
    }
}
