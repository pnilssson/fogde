use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::agents::Agent;
use crate::{Event, Method};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Terminal {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub checkout: Option<String>,
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

pub struct FrameEvent;

impl Event for FrameEvent {
    const NAME: &'static str = "terminals.frame";
    type Params = TerminalFrame;
}

pub struct EndedEvent;

impl Event for EndedEvent {
    const NAME: &'static str = "terminals.ended";
    type Params = TerminalEnded;
}
