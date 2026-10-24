use std::sync::Arc;
use std::time::{Duration, SystemTime};

use agents::limits;
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
        if polled.is_some_and(|(at, last)| last == shown && at.elapsed() < POLL) {
            continue;
        }

        polled = Some((Instant::now(), shown));
        let (claude, codex, copilot) = tokio::join!(
            async {
                if shown.claude {
                    claude().await
                } else {
                    Vec::new()
                }
            },
            async {
                if shown.codex {
                    codex().await
                } else {
                    Vec::new()
                }
            },
            async {
                if shown.copilot {
                    copilot(&shared).await
                } else {
                    Vec::new()
                }
            },
        );
        let at = seconds(SystemTime::now());
        shared.usage.set(Agent::Claude, claude, at);
        shared.usage.set(Agent::Codex, codex, at);
        shared.usage.set(Agent::Copilot, copilot, at);
    }
}

async fn claude() -> Vec<Window> {
    match limits::claude().await {
        Ok(limits) => windows(limits),
        Err(error) => {
            tracing::debug!(%error, "could not read the Claude limits");
            Vec::new()
        }
    }
}

async fn codex() -> Vec<Window> {
    match limits::codex().await {
        Ok(limits) => windows(limits),
        Err(error) => {
            tracing::debug!(%error, "could not read the Codex limits");
            Vec::new()
        }
    }
}

fn windows(limits: Vec<limits::Limit>) -> Vec<Window> {
    limits
        .into_iter()
        .map(|limit| window(span(limit.minutes), 100.0 - limit.used, limit.resets))
        .collect()
}

async fn copilot(shared: &Shared) -> Vec<Window> {
    match agents::Agent::Copilot.state().await {
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
