use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::terminals::Terminal;

#[derive(Clone, Debug, Serialize, TS)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder: String,
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
pub struct RenameWorkspaceParams {
    pub workspace: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct PlaceTerminalParams {
    pub terminal: String,
    pub workspace: String,
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

pub struct RenameWorkspace;

impl Method for RenameWorkspace {
    const NAME: &'static str = "workspaces.rename";
    type Params = RenameWorkspaceParams;
    type Result = ();
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
