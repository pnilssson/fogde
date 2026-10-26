mod events;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::agents::Agent;

pub use events::{
    EndedEvent, FailedEvent, FrameEvent, ScrolledEvent, StateEvent, TerminalEnded, TerminalFailed,
    TerminalFrame, TerminalScrolled,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Terminal {
    pub id: String,
    pub name: String,
    pub agent: Option<Agent>,
    pub status: Status,
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
    pub status: Status,
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
#[serde(rename_all = "camelCase")]
pub enum Key {
    ShiftEnter,
    CtrlEnter,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct TerminalKeyParams {
    pub id: String,
    pub key: Key,
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

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ClickTerminalParams {
    pub id: String,
    pub column: u16,
    pub row: u16,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct SeekTerminalParams {
    pub id: String,
    pub offset: u32,
}

pub const IMAGE_LIMIT: usize = 10 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Image {
    Png,
    Jpeg,
    Gif,
    Webp,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct PasteImageParams {
    pub id: String,
    pub image: Image,
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

pub struct TerminalKey;

impl Method for TerminalKey {
    const NAME: &'static str = "terminals.key";
    type Params = TerminalKeyParams;
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

pub struct ClickTerminal;

impl Method for ClickTerminal {
    const NAME: &'static str = "terminals.click";
    type Params = ClickTerminalParams;
    type Result = ();
}

pub struct SeekTerminal;

impl Method for SeekTerminal {
    const NAME: &'static str = "terminals.seek";
    type Params = SeekTerminalParams;
    type Result = ();
}

pub struct PasteImage;

impl Method for PasteImage {
    const NAME: &'static str = "terminals.pasteImage";
    type Params = PasteImageParams;
    type Result = ();
}

pub struct ViewTerminal;

impl Method for ViewTerminal {
    const NAME: &'static str = "terminals.view";
    type Params = Option<String>;
    type Result = ();
}
