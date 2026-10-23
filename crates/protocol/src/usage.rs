use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::agents::Agent;
use crate::{Event, Method};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Span {
    Session,
    Week,
    Month,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
pub struct Window {
    pub span: Span,
    pub left: f64,
    #[ts(type = "number")]
    pub resets: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct Usage {
    pub agent: Agent,
    pub windows: Vec<Window>,
    #[ts(type = "number")]
    pub at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Shown {
    pub claude: bool,
    pub codex: bool,
    pub copilot: bool,
}

impl Default for Shown {
    fn default() -> Self {
        Self {
            claude: true,
            codex: true,
            copilot: true,
        }
    }
}

pub struct ListUsage;

impl Method for ListUsage {
    const NAME: &'static str = "usage.list";
    type Params = ();
    type Result = Vec<Usage>;
}

pub struct GetShown;

impl Method for GetShown {
    const NAME: &'static str = "usage.getShown";
    type Params = ();
    type Result = Shown;
}

pub struct SetShown;

impl Method for SetShown {
    const NAME: &'static str = "usage.setShown";
    type Params = Shown;
    type Result = ();
}

pub struct UsageEvent;

impl Event for UsageEvent {
    const NAME: &'static str = "usage.changed";
    type Params = Vec<Usage>;
}
