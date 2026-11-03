mod events;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::agents::Agent;

pub use events::{EndedEvent, OutputEvent, StateEvent, TerminalEnded, TerminalOutput};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Terminal {
    pub id: String,
    pub name: String,
    pub agent: Option<Agent>,
    pub model: Option<String>,
    pub status: Status,
    #[ts(type = "number | null")]
    pub active: Option<i64>,
    pub folder: String,
    pub branch: Option<String>,
    pub changes: Option<Changes>,
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
    pub model: Option<String>,
    pub status: Status,
    #[ts(type = "number | null")]
    pub active: Option<i64>,
    pub folder: String,
    pub branch: Option<String>,
    pub changes: Option<Changes>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Changes {
    pub added: u32,
    pub removed: u32,
    pub base: Option<Base>,
    pub upstream: Option<Divergence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Base {
    pub name: String,
    pub behind: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Divergence {
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CreateTerminalParams {
    pub workspace: String,
    pub folder: String,
    pub agent: Option<Agent>,
    pub size: Option<Size>,
}

#[derive(Clone, Copy, Debug, Deserialize, TS)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
pub struct Display {
    pub fg: String,
    pub bg: String,
    pub cursor: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct WatchTerminalParams {
    pub id: String,
    pub size: Size,
    pub display: Display,
    pub token: u32,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ResizeTerminalParams {
    pub id: String,
    pub size: Size,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct DisplayTerminalParams {
    pub id: String,
    pub display: Display,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct TerminalInputParams {
    pub id: String,
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct FocusTerminalParams {
    pub id: String,
    pub focused: bool,
}

pub const FILE_LIMIT: usize = 10 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, TS)]
pub struct PasteFileParams {
    pub id: String,
    pub name: String,
    pub data: String,
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

pub struct TerminalReply;

impl Method for TerminalReply {
    const NAME: &'static str = "terminals.reply";
    type Params = TerminalInputParams;
    type Result = ();
}

pub struct FocusTerminal;

impl Method for FocusTerminal {
    const NAME: &'static str = "terminals.focus";
    type Params = FocusTerminalParams;
    type Result = ();
}

pub struct ResizeTerminal;

impl Method for ResizeTerminal {
    const NAME: &'static str = "terminals.resize";
    type Params = ResizeTerminalParams;
    type Result = ();
}

pub struct DisplayTerminal;

impl Method for DisplayTerminal {
    const NAME: &'static str = "terminals.display";
    type Params = DisplayTerminalParams;
    type Result = ();
}

pub struct PasteFile;

impl Method for PasteFile {
    const NAME: &'static str = "terminals.pasteFile";
    type Params = PasteFileParams;
    type Result = ();
}

pub struct ViewTerminal;

impl Method for ViewTerminal {
    const NAME: &'static str = "terminals.view";
    type Params = Option<String>;
    type Result = ();
}
