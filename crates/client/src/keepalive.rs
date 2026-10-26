#[cfg(test)]
mod tests;

use std::pin::pin;
use std::sync::Weak;
use std::time::Duration;

use protocol::health::Ping;
use tokio::sync::watch;
use tokio::time::{sleep, timeout};

use crate::client::{Client, Inner, State};

pub(crate) struct Keepalive {
    pub interval: Duration,
    pub deadline: Duration,
    pub first: Duration,
    pub ceiling: Duration,
    pub random: fn() -> f64,
}

impl Default for Keepalive {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(15),
            deadline: Duration::from_secs(10),
            first: Duration::from_secs(1),
            ceiling: Duration::from_secs(30),
            random,
        }
    }
}

pub(crate) trait Watched: Send + Sync + 'static {
    fn buffered(&self) -> usize;
    fn ping(&self) -> impl Future<Output = ()> + Send;
    fn lose(&self, error: &str);
    fn reconnect(&self);
}

pub(crate) struct Handle(pub Weak<Inner>);

impl Handle {
    fn client(&self) -> Option<Client> {
        self.0.upgrade().map(Client)
    }
}

impl Watched for Handle {
    fn buffered(&self) -> usize {
        self.client().map_or(0, |client| client.buffered())
    }

    async fn ping(&self) {
        if let Some(client) = self.client() {
            let _ = client.call::<Ping>(()).await;
        }
    }

    fn lose(&self, error: &str) {
        if let Some(client) = self.client() {
            client.lose(error);
        }
    }

    fn reconnect(&self) {
        if let Some(client) = self.client() {
            tokio::spawn(async move {
                let _ = client.reconnect().await;
            });
        }
    }
}

pub(crate) async fn run<W: Watched>(
    watched: W,
    mut states: watch::Receiver<State>,
    timing: Keepalive,
) {
    let mut attempt = 0;
    loop {
        let state = states.borrow_and_update().clone();
        match state {
            State::Connected => {
                attempt = 0;
                tokio::select! {
                    changed = states.changed() => if changed.is_err() {
                        return;
                    },
                    () = alive(&watched, &timing) => {
                        if states.changed().await.is_err() {
                            return;
                        }
                    }
                }
            }
            State::Disconnected {
                retryable: true, ..
            } => {
                let delay = backoff(&timing, attempt);
                attempt += 1;
                tokio::select! {
                    changed = states.changed() => if changed.is_err() {
                        return;
                    },
                    () = sleep(delay) => {
                        watched.reconnect();
                        if states.changed().await.is_err() {
                            return;
                        }
                    }
                }
            }
            State::Idle | State::Connecting | State::Disconnected { .. } => {
                if states.changed().await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn alive<W: Watched>(watched: &W, timing: &Keepalive) {
    loop {
        sleep(timing.interval).await;

        let mut ping = pin!(watched.ping());
        let mut left = watched.buffered();
        loop {
            if timeout(timing.deadline, &mut ping).await.is_ok() {
                break;
            }

            let before = left;
            left = watched.buffered();
            if before == 0 || left >= before {
                watched.lose("The connection stopped responding.");
                return;
            }
        }
    }
}

fn backoff(timing: &Keepalive, attempt: u32) -> Duration {
    let delay = timing
        .first
        .saturating_mul(1 << attempt.min(16))
        .min(timing.ceiling);

    delay.mul_f64(0.5 + (timing.random)() / 2.0)
}

fn random() -> f64 {
    getrandom::u32().map_or(1.0, |value| f64::from(value) / f64::from(u32::MAX))
}
