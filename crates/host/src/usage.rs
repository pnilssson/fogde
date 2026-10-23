mod claude;
mod poll;
mod statusline;

use std::future;
use std::sync::{Arc, Once};
use std::time::{SystemTime, UNIX_EPOCH};

use protocol::agents::Agent;
use protocol::rpc::Failure;
use protocol::usage::{GetShown, ListUsage, SetShown, Shown, Span, Usage, Window};
use tokio::sync::watch;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
use crate::shared::Shared;

pub use claude::{launch, report};

pub struct Meter {
    usage: watch::Sender<Vec<Usage>>,
    started: Once,
}

impl Default for Meter {
    fn default() -> Self {
        Self {
            usage: watch::Sender::new(Vec::new()),
            started: Once::new(),
        }
    }
}

impl Meter {
    fn set(&self, agent: Agent, windows: Vec<Window>, at: i64) {
        self.usage.send_if_modified(|list| {
            let index = list.iter().position(|usage| usage.agent == agent);
            let usage = Usage { agent, windows, at };
            match index {
                Some(index) if list[index] == usage => false,
                Some(index) if usage.windows.is_empty() => {
                    list.remove(index);
                    true
                }
                Some(index) => {
                    list[index] = usage;
                    true
                }
                None if usage.windows.is_empty() => false,
                None => {
                    list.push(usage);
                    true
                }
            }
        });
    }

    fn watching(&self) -> usize {
        self.usage.receiver_count()
    }
}

pub fn changes(shared: &Arc<Shared>) -> watch::Receiver<Vec<Usage>> {
    shared.usage.started.call_once(|| {
        tokio::spawn(poll::claude(Arc::clone(shared)));
        tokio::spawn(poll::others(Arc::clone(shared)));
    });

    shared.usage.usage.subscribe()
}

pub fn register(router: &mut Router) {
    router.add::<ListUsage>();
    router.add::<GetShown>();
    router.add::<SetShown>();
}

impl Handler for ListUsage {
    fn handle(
        session: &Session,
        (): (),
    ) -> impl Future<Output = Result<Vec<Usage>, Failure>> + Send {
        future::ready(Ok(changes(&session.shared).borrow().clone()))
    }
}

impl Handler for GetShown {
    async fn handle(session: &Session, (): ()) -> Result<Shown, Failure> {
        session.shared.settings.shown().await.map_err(failure)
    }
}

impl Handler for SetShown {
    async fn handle(session: &Session, shown: Shown) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_shown(shown)
            .await
            .map_err(failure)
    }
}

fn span(minutes: i64) -> Span {
    match minutes {
        ..=1440 => Span::Session,
        1441..=10080 => Span::Week,
        _ => Span::Month,
    }
}

fn window(span: Span, left: f64, resets: i64) -> Window {
    Window {
        span,
        left: left.clamp(0.0, 100.0),
        resets,
    }
}

fn seconds(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refreshes_an_unchanged_reading() {
        let meter = Meter::default();
        let windows = vec![window(Span::Session, 40.0, 1_738_425_600)];

        meter.set(Agent::Claude, windows.clone(), 100);
        meter.set(Agent::Claude, windows.clone(), 200);

        assert_eq!(
            *meter.usage.borrow(),
            [Usage {
                agent: Agent::Claude,
                windows,
                at: 200,
            }]
        );
    }
}
