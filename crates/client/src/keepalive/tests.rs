use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::watch;

use super::{Keepalive, Watched, run};
use crate::client::State;

struct Fake {
    state: watch::Sender<State>,
    silent: AtomicBool,
    pings: AtomicUsize,
    reconnects: AtomicUsize,
    lost: Mutex<Vec<String>>,
    queued: Mutex<VecDeque<usize>>,
}

impl Fake {
    fn new() -> Self {
        Self {
            state: watch::Sender::new(State::Idle),
            silent: AtomicBool::default(),
            pings: AtomicUsize::default(),
            reconnects: AtomicUsize::default(),
            lost: Mutex::default(),
            queued: Mutex::default(),
        }
    }

    fn set(&self, state: State) {
        self.state.send_replace(state);
    }

    fn pings(&self) -> usize {
        self.pings.load(Ordering::SeqCst)
    }

    fn reconnects(&self) -> usize {
        self.reconnects.load(Ordering::SeqCst)
    }

    fn lost(&self) -> Vec<String> {
        self.lost.lock().unwrap().clone()
    }
}

impl Watched for Arc<Fake> {
    fn buffered(&self) -> usize {
        self.queued.lock().unwrap().pop_front().unwrap_or(0)
    }

    async fn ping(&self) {
        self.pings.fetch_add(1, Ordering::SeqCst);
        if self.silent.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
    }

    fn lose(&self, error: &str) {
        self.lost.lock().unwrap().push(error.to_owned());
        self.set(disconnected(true));
    }

    fn reconnect(&self) {
        self.reconnects.fetch_add(1, Ordering::SeqCst);
    }
}

fn disconnected(retryable: bool) -> State {
    State::Disconnected {
        error: "The connection was lost.".to_owned(),
        retryable,
    }
}

fn start() -> Arc<Fake> {
    let fake = Arc::new(Fake::new());
    let timing = Keepalive {
        random: || 1.0,
        ..Keepalive::default()
    };
    tokio::spawn(run(fake.clone(), fake.state.subscribe(), timing));
    fake
}

async fn advance(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}

async fn set(fake: &Fake, state: State) {
    fake.set(state);
    advance(0).await;
}

#[tokio::test(start_paused = true)]
async fn pings_a_connected_host_every_15_seconds() {
    let fake = start();
    set(&fake, State::Connected).await;

    advance(14_999).await;
    assert_eq!(fake.pings(), 0);
    advance(1).await;
    assert_eq!(fake.pings(), 1);
    advance(15_000).await;
    assert_eq!(fake.pings(), 2);
    assert_eq!(fake.lost(), Vec::<String>::new());
}

#[tokio::test(start_paused = true)]
async fn drops_a_connection_that_stops_answering_and_reconnects() {
    let fake = start();
    set(&fake, State::Connected).await;
    fake.silent.store(true, Ordering::SeqCst);

    advance(15_000 + 9_999).await;
    assert_eq!(fake.lost(), Vec::<String>::new());
    advance(1).await;
    assert_eq!(fake.lost(), ["The connection stopped responding."]);

    advance(1_000).await;
    assert_eq!(fake.reconnects(), 1);
}

#[tokio::test(start_paused = true)]
async fn waits_for_a_large_upload_to_go_out_before_giving_up_on_the_ping() {
    let fake = start();
    set(&fake, State::Connected).await;
    fake.silent.store(true, Ordering::SeqCst);
    fake.queued
        .lock()
        .unwrap()
        .extend([9_000_000, 6_000_000, 2_000_000, 0]);

    advance(15_000 + 30_000).await;
    assert_eq!(fake.lost(), Vec::<String>::new());
    advance(10_000).await;
    assert_eq!(fake.lost(), ["The connection stopped responding."]);
}

#[tokio::test(start_paused = true)]
async fn drops_a_connection_whose_upload_stops_moving() {
    let fake = start();
    set(&fake, State::Connected).await;
    fake.silent.store(true, Ordering::SeqCst);
    fake.queued.lock().unwrap().extend([9_000_000, 9_000_000]);

    advance(15_000 + 10_000).await;
    assert_eq!(fake.lost(), ["The connection stopped responding."]);
}

#[tokio::test(start_paused = true)]
async fn retries_a_lost_connection_with_growing_delays_up_to_30_seconds() {
    let fake = start();
    let mut waits = Vec::new();

    for attempt in 0..7 {
        set(&fake, disconnected(true)).await;
        let mut waited = 0;
        while fake.reconnects() == attempt {
            advance(1_000).await;
            waited += 1_000;
        }
        waits.push(waited);
        set(&fake, State::Connecting).await;
    }

    assert_eq!(waits, [1_000, 2_000, 4_000, 8_000, 16_000, 30_000, 30_000]);
}

#[tokio::test(start_paused = true)]
async fn starts_over_after_a_successful_connection() {
    let fake = start();
    for _ in 0..3 {
        set(&fake, disconnected(true)).await;
        advance(30_000).await;
        set(&fake, State::Connecting).await;
    }
    set(&fake, State::Connected).await;
    set(&fake, disconnected(true)).await;

    let before = fake.reconnects();
    advance(1_000).await;
    assert_eq!(fake.reconnects(), before + 1);
}

#[tokio::test(start_paused = true)]
async fn never_retries_a_failure_the_user_must_fix_or_a_reset_connection() {
    let fake = start();
    set(&fake, disconnected(false)).await;
    advance(60_000).await;
    assert_eq!(fake.reconnects(), 0);

    set(&fake, State::Connected).await;
    set(&fake, State::Idle).await;
    advance(60_000).await;
    assert_eq!(fake.pings(), 0);
    assert_eq!(fake.reconnects(), 0);
}
