use serde::Serialize;
use ts_rs::TS;

use super::TerminalState;
use crate::Event;

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalOutput {
    pub id: String,
    pub token: u32,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalEnded {
    pub id: String,
    pub error: Option<String>,
}

pub struct OutputEvent;

impl Event for OutputEvent {
    const NAME: &'static str = "terminals.output";
    type Params = TerminalOutput;
}

pub struct StateEvent;

impl Event for StateEvent {
    const NAME: &'static str = "terminals.state";
    type Params = TerminalState;
}

pub struct EndedEvent;

impl Event for EndedEvent {
    const NAME: &'static str = "terminals.ended";
    type Params = TerminalEnded;
}
