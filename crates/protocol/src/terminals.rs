use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::agents::Agent;
use crate::{Event, Method};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Terminal {
    pub id: String,
    pub name: String,
    pub agent: Option<Agent>,
    pub status: Status,
    pub folder: String,
    pub checkout: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Idle,
    Working,
    Blocked,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct TerminalState {
    pub id: String,
    pub name: String,
    pub agent: Option<Agent>,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CreateTerminalParams {
    pub workspace: String,
    pub folder: String,
    pub agent: Option<Agent>,
}

#[derive(Clone, Copy, Debug, Deserialize, TS)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct WatchTerminalParams {
    pub id: String,
    pub size: Size,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct TerminalInputParams {
    pub id: String,
    pub data: String,
}

#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Scroll {
    Up,
    Down,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ScrollTerminalParams {
    pub id: String,
    pub direction: Scroll,
    pub lines: u16,
    pub column: u16,
    pub row: u16,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct TerminalFrame {
    pub id: String,
    pub cols: u16,
    pub rows: u16,
    pub full: bool,
    pub data: String,
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

pub struct CreateTerminal;

impl Method for CreateTerminal {
    const NAME: &'static str = "terminals.create";
    type Params = CreateTerminalParams;
    type Result = Terminal;
}

pub struct CloseTerminal;

impl Method for CloseTerminal {
    const NAME: &'static str = "terminals.close";
    type Params = String;
    type Result = ();
}

pub struct WatchTerminal;

impl Method for WatchTerminal {
    const NAME: &'static str = "terminals.watch";
    type Params = WatchTerminalParams;
    type Result = ();
}

pub struct UnwatchTerminal;

impl Method for UnwatchTerminal {
    const NAME: &'static str = "terminals.unwatch";
    type Params = String;
    type Result = ();
}

pub struct TerminalInput;

impl Method for TerminalInput {
    const NAME: &'static str = "terminals.input";
    type Params = TerminalInputParams;
    type Result = ();
}

pub struct ResizeTerminal;

impl Method for ResizeTerminal {
    const NAME: &'static str = "terminals.resize";
    type Params = WatchTerminalParams;
    type Result = ();
}

pub struct ScrollTerminal;

impl Method for ScrollTerminal {
    const NAME: &'static str = "terminals.scroll";
    type Params = ScrollTerminalParams;
    type Result = ();
}

pub struct ViewTerminal;

impl Method for ViewTerminal {
    const NAME: &'static str = "terminals.view";
    type Params = Option<String>;
    type Result = ();
}

pub struct FrameEvent;

impl Event for FrameEvent {
    const NAME: &'static str = "terminals.frame";
    type Params = TerminalFrame;
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
