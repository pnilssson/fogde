use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::terminals::Terminal;
use crate::{Event, Method};

#[derive(Clone, Debug, Serialize, TS)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub missing: bool,
    pub checkouts: Vec<Checkout>,
    pub terminals: Vec<Terminal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Checkout {
    pub path: String,
    pub branch: Option<String>,
    pub main: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct PlaceTerminalParams {
    pub terminal: String,
    pub workspace: String,
    pub checkout: Option<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct MoveTerminalParams {
    pub terminal: String,
    pub before: Option<String>,
}

pub struct ListWorkspaces;

impl Method for ListWorkspaces {
    const NAME: &'static str = "workspaces.list";
    type Params = ();
    type Result = Vec<Workspace>;
}

pub struct CreateWorkspace;

impl Method for CreateWorkspace {
    const NAME: &'static str = "workspaces.create";
    type Params = String;
    type Result = Workspace;
}

pub struct DeleteWorkspace;

impl Method for DeleteWorkspace {
    const NAME: &'static str = "workspaces.delete";
    type Params = String;
    type Result = ();
}

pub struct PlaceTerminal;

impl Method for PlaceTerminal {
    const NAME: &'static str = "workspaces.place";
    type Params = PlaceTerminalParams;
    type Result = ();
}

pub struct MoveTerminal;

impl Method for MoveTerminal {
    const NAME: &'static str = "workspaces.move";
    type Params = MoveTerminalParams;
    type Result = ();
}

pub struct ChangedEvent;

impl Event for ChangedEvent {
    const NAME: &'static str = "workspaces.changed";
    type Params = ();
}
