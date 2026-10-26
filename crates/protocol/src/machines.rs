use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Machine {
    Computer {
        name: String,
        connected: bool,
    },
    Server {
        id: String,
        name: String,
        target: String,
        connected: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MachineId {
    Computer,
    Server { id: String },
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct RenameMachineParams {
    pub machine: MachineId,
    pub name: String,
}

pub struct ListMachines;

impl Method for ListMachines {
    const NAME: &'static str = "machines.list";
    type Params = ();
    type Result = Vec<Machine>;
}

pub struct AddComputer;

impl Method for AddComputer {
    const NAME: &'static str = "machines.addComputer";
    type Params = ();
    type Result = ();
}

pub struct ConnectMachine;

impl Method for ConnectMachine {
    const NAME: &'static str = "machines.connect";
    type Params = MachineId;
    type Result = ();
}

pub struct DisconnectMachine;

impl Method for DisconnectMachine {
    const NAME: &'static str = "machines.disconnect";
    type Params = MachineId;
    type Result = ();
}

pub struct RenameMachine;

impl Method for RenameMachine {
    const NAME: &'static str = "machines.rename";
    type Params = RenameMachineParams;
    type Result = ();
}

pub struct RemoveMachine;

impl Method for RemoveMachine {
    const NAME: &'static str = "machines.remove";
    type Params = MachineId;
    type Result = ();
}
