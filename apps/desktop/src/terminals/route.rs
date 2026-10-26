use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use client::Client;
use gpui_kit::{App, Global, SharedString, Task};
use protocol::Event;
use protocol::terminals::{
    EndedEvent, FrameEvent, ScrolledEvent, TerminalEnded, TerminalFrame, TerminalScrolled,
};
use tokio::sync::mpsc;

use crate::runtime;

pub enum Routed {
    Frame(TerminalFrame),
    Scrolled(TerminalScrolled),
    Ended(TerminalEnded),
}

#[derive(Clone, Default)]
pub struct Table(Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Routed>>>>);

impl Table {
    pub fn subscribe(&self, id: &str) -> mpsc::UnboundedReceiver<Routed> {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id.to_owned(), sender);
        receiver
    }

    pub fn forward(&self, id: &str, routed: Routed) {
        let mut table = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(sender) = table.get(id) else {
            return;
        };
        if sender.send(routed).is_err() {
            table.remove(id);
        }
    }
}

struct Router {
    client: Client,
    table: Table,
    _tasks: Vec<Task<Option<()>>>,
}

#[derive(Default)]
struct Routers(HashMap<SharedString, Router>);

impl Global for Routers {}

fn listen<E: Event + 'static>(
    client: &Client,
    table: &Table,
    cx: &App,
    route: fn(E::Params) -> (String, Routed),
) -> Task<Option<()>>
where
    E::Params: Send,
{
    let mut listener = client.listen::<E>();
    let table = table.clone();
    runtime::spawn(cx, async move {
        while let Some(params) = listener.recv().await {
            let (id, routed) = route(params);
            table.forward(&id, routed);
        }
    })
}

fn router(client: &Client, cx: &App) -> Router {
    let table = Table::default();
    let tasks = vec![
        listen::<FrameEvent>(client, &table, cx, |frame| {
            (frame.id.clone(), Routed::Frame(frame))
        }),
        listen::<ScrolledEvent>(client, &table, cx, |scrolled| {
            (scrolled.id.clone(), Routed::Scrolled(scrolled))
        }),
        listen::<EndedEvent>(client, &table, cx, |ended| {
            (ended.id.clone(), Routed::Ended(ended))
        }),
    ];
    Router {
        client: client.clone(),
        table,
        _tasks: tasks,
    }
}

pub fn subscribe(
    machine: &SharedString,
    client: &Client,
    id: &str,
    cx: &mut App,
) -> mpsc::UnboundedReceiver<Routed> {
    let current = cx
        .try_global::<Routers>()
        .and_then(|routers| routers.0.get(machine))
        .is_some_and(|router| router.client.same(client));
    if !current {
        let fresh = router(client, cx);
        cx.default_global::<Routers>()
            .0
            .insert(machine.clone(), fresh);
    }
    cx.global::<Routers>().0[machine].table.subscribe(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ended(id: &str) -> Routed {
        Routed::Ended(TerminalEnded {
            id: id.to_owned(),
            error: None,
        })
    }

    fn id(routed: &Routed) -> &str {
        match routed {
            Routed::Frame(frame) => &frame.id,
            Routed::Scrolled(scrolled) => &scrolled.id,
            Routed::Ended(ended) => &ended.id,
        }
    }

    #[test]
    fn each_terminal_receives_only_its_own_events() {
        let table = Table::default();
        let mut first = table.subscribe("w1");
        let mut second = table.subscribe("w2");

        table.forward("w2", ended("w2"));
        table.forward("w1", ended("w1"));
        table.forward("w3", ended("w3"));

        assert_eq!(first.try_recv().as_ref().map(id).ok(), Some("w1"));
        assert!(first.try_recv().is_err());
        assert_eq!(second.try_recv().as_ref().map(id).ok(), Some("w2"));
        assert!(second.try_recv().is_err());
    }

    #[test]
    fn a_closed_terminal_is_dropped_from_the_table() {
        let table = Table::default();
        drop(table.subscribe("w1"));

        table.forward("w1", ended("w1"));

        assert!(table.0.lock().unwrap().is_empty());
    }
}
