use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Stats {
    pub behind: bool,
    pub lines: bool,
    pub upstream: bool,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            behind: true,
            lines: true,
            upstream: false,
        }
    }
}

pub struct GetStats;

impl Method for GetStats {
    const NAME: &'static str = "stats.get";
    type Params = ();
    type Result = Stats;
}

pub struct SetStats;

impl Method for SetStats {
    const NAME: &'static str = "stats.set";
    type Params = Stats;
    type Result = ();
}
