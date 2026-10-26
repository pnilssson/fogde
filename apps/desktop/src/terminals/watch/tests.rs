use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use protocol::terminals::Size;
use tokio::sync::Notify;

use super::{Calls, Queue, Watch};

const SIZE: Size = Size { cols: 80, rows: 24 };

#[derive(Clone, Default)]
struct Fake {
    calls: Arc<Mutex<Vec<&'static str>>>,
    hold: Option<Arc<Notify>>,
    stalled: bool,
}

impl Calls for Fake {
    fn watch(&self, _id: String, _size: Size) -> BoxFuture<'static, Result<(), client::Error>> {
        let first = {
            let mut calls = self.calls.lock().unwrap();
            calls.push("terminals.watch");
            calls.len() == 1
        };
        let hold = self.hold.clone().filter(|_| first);
        let stalled = self.stalled;
        async move {
            if stalled {
                std::future::pending::<()>().await;
            }
            if let Some(hold) = hold {
                hold.notified().await;
            }
            Ok(())
        }
        .boxed()
    }

    fn unwatch(&self, _id: String) -> BoxFuture<'static, Result<(), client::Error>> {
        self.calls.lock().unwrap().push("terminals.unwatch");
        async { Ok(()) }.boxed()
    }

    fn connected(&self) -> bool {
        !self.stalled
    }
}

#[tokio::test]
async fn a_stalled_server_watch_does_not_block_a_local_terminal() {
    let remote = Fake {
        stalled: true,
        ..Fake::default()
    };
    let (_remote, _, tail) = Watch::start(remote, Queue::default(), "w1".into(), SIZE);
    tokio::spawn(tail);

    let (_local, ready, tail) = Watch::start(Fake::default(), Queue::default(), "w1".into(), SIZE);
    tokio::spawn(tail);

    let result = tokio::time::timeout(Duration::from_millis(500), ready).await;
    assert!(matches!(result, Ok(Ok(Ok(())))));
}

#[tokio::test]
async fn a_late_cleanup_finishes_before_the_next_watch_on_the_same_machine() {
    let hold = Arc::new(Notify::new());
    let fake = Fake {
        hold: Some(hold.clone()),
        ..Fake::default()
    };
    let queue = Queue::default();

    let (first, stale, tail) = Watch::start(fake.clone(), queue.clone(), "w1".into(), SIZE);
    tokio::spawn(tail);
    tokio::task::yield_now().await;

    if let Some(tail) = first.stop(true) {
        tokio::spawn(tail);
    }
    let (_second, ready, tail) = Watch::start(fake.clone(), queue, "w1".into(), SIZE);
    tokio::spawn(tail);
    hold.notify_one();

    assert!(matches!(ready.await, Ok(Ok(()))));
    assert!(stale.await.is_err());
    assert_eq!(
        *fake.calls.lock().unwrap(),
        ["terminals.watch", "terminals.unwatch", "terminals.watch"]
    );
}
