use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Shell {
    pub program: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ShellOption {
    pub name: String,
    pub program: String,
}

pub struct GetShell;

impl Method for GetShell {
    const NAME: &'static str = "shell.get";
    type Params = ();
    type Result = Shell;
}

pub struct SetShell;

impl Method for SetShell {
    const NAME: &'static str = "shell.set";
    type Params = Shell;
    type Result = ();
}

pub struct ListShells;

impl Method for ListShells {
    const NAME: &'static str = "shell.list";
    type Params = ();
    type Result = Vec<ShellOption>;
}
