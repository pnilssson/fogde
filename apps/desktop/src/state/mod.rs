pub mod agents;
pub mod cache;
pub mod listen;
pub mod machines;
pub mod network;
pub mod notes;
pub mod settings;
pub mod workspaces;

use client::Client;
use gpui_kit::{App, AppContext, Entity, Global};
use protocol::Connection;

use agents::Agents;
use machines::Machines;
use notes::Notes;
use settings::Settings;
use workspaces::Workspaces;

pub struct Store {
    pub machines: Entity<Machines>,
    pub workspaces: Entity<Workspaces>,
    pub agents: Entity<Agents>,
    pub settings: Entity<Settings>,
    pub notes: Entity<Notes>,
}

impl Global for Store {}

pub fn init(own: Client, connection: Connection, cx: &mut App) {
    let settings = cx.new(|cx| Settings::new(own.clone(), cx));
    let machines = cx.new(|cx| Machines::new(own, connection, cx));
    let workspaces = cx.new(|cx| Workspaces::new(machines.clone(), cx));
    let agents = cx.new(|cx| Agents::new(machines.clone(), cx));
    let notes = cx.new(|cx| Notes::new(machines.clone(), cx));
    cx.set_global(Store {
        machines,
        workspaces,
        agents,
        settings,
        notes,
    });
}

pub fn store(cx: &App) -> &Store {
    cx.global::<Store>()
}
