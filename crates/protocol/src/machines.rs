use serde::Serialize;
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Machine {
    Computer,
    Server { id: String, target: String },
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

pub struct RemoveComputer;

impl Method for RemoveComputer {
    const NAME: &'static str = "machines.removeComputer";
    type Params = ();
    type Result = ();
}
