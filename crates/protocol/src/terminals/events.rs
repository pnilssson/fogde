use serde::Serialize;
use ts_rs::TS;

use super::TerminalState;
use crate::Event;
use crate::agents::Agent;

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalFrame {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
    pub full: bool,
    pub data: String,
    pub viewport: Option<Viewport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Viewport {
    pub offset: u32,
    pub max: u32,
    pub rows: u32,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalScrolled {
    pub id: String,
    #[serde(flatten)]
    pub viewport: Viewport,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalEnded {
    pub id: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalFailed {
    pub id: String,
    pub agent: Agent,
    pub error: String,
}

pub struct FrameEvent;

impl Event for FrameEvent {
    const NAME: &'static str = "terminals.frame";
    type Params = TerminalFrame;
}

pub struct ScrolledEvent;

impl Event for ScrolledEvent {
    const NAME: &'static str = "terminals.scrolled";
    type Params = TerminalScrolled;
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

pub struct FailedEvent;

impl Event for FailedEvent {
    const NAME: &'static str = "terminals.failed";
    type Params = TerminalFailed;
}
