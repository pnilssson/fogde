use gpui_kit::{App, AppContext, Context, Subscription};
use protocol::terminals::ViewTerminal;

use super::Desk;
use crate::runtime;
use crate::state::machines::Change;
use crate::state::store;
use crate::workspaces::tiles::tile::Selection;

pub struct Viewing {
    current: Option<Selection>,
    _connected: Option<Subscription>,
}

impl Viewing {
    pub fn new(cx: &mut Context<Desk>) -> Self {
        let machines = store(cx).machines.clone();
        let subscription = cx.subscribe(&machines, |desk, _, change, cx| {
            if let Change::Connected(machine) = change {
                desk.viewing.send(machine, cx);
            }
        });
        Self {
            current: None,
            _connected: Some(subscription),
        }
    }

    pub fn set(&mut self, next: Option<Selection>, cx: &mut App) {
        if self.current == next {
            return;
        }
        self.current = next;

        let machines: Vec<String> = store(cx)
            .machines
            .read(cx)
            .connected()
            .map(|machine| machine.id.to_string())
            .collect();
        for machine in machines {
            self.send(&machine, cx);
        }
    }

    fn send(&self, machine: &str, cx: &mut App) {
        let state = store(cx).machines.read(cx);
        let Some(client) = state.get(machine).map(|machine| machine.client.clone()) else {
            return;
        };
        if state.state(machine) != client::State::Connected {
            return;
        }

        let id = self
            .current
            .as_ref()
            .filter(|selection| selection.machine == machine)
            .map(|selection| selection.id.clone());
        let request = runtime::call::<ViewTerminal>(&client, id, cx);
        cx.background_spawn(async move {
            let _ = request.await;
        })
        .detach();
    }
}
