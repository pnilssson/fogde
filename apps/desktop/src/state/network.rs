use std::collections::BTreeSet;
use std::net::IpAddr;
use std::time::Duration;

use gpui_kit::{App, Entity, Task};
use tokio::sync::mpsc;

use super::machines::Machines;
use crate::runtime;

const EVERY: Duration = Duration::from_secs(5);

pub struct Network {
    _poll: Task<Option<()>>,
    _retry: Task<()>,
}

fn addresses() -> BTreeSet<IpAddr> {
    if_addrs::get_if_addrs()
        .map(|found| {
            found
                .into_iter()
                .filter(|address| !address.is_loopback())
                .map(|address| address.ip())
                .collect()
        })
        .unwrap_or_default()
}

pub fn watch(machines: &Entity<Machines>, cx: &mut App) -> Network {
    let (sender, mut changes) = mpsc::channel(1);
    let poll = runtime::spawn(cx, async move {
        let mut known = tokio::task::spawn_blocking(addresses)
            .await
            .unwrap_or_default();
        loop {
            tokio::time::sleep(EVERY).await;
            let now = tokio::task::spawn_blocking(addresses)
                .await
                .unwrap_or_default();
            let returned = now != known && !now.is_empty();
            known = now;
            if returned && sender.send(()).await.is_err() {
                return;
            }
        }
    });
    let machines = machines.downgrade();
    let retry = cx.spawn(async move |cx| {
        while changes.recv().await.is_some() {
            if machines
                .update(cx, |machines, cx| machines.retry(cx))
                .is_err()
            {
                return;
            }
        }
    });
    Network {
        _poll: poll,
        _retry: retry,
    }
}
