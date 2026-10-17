use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Serialize, TS)]
pub struct Frame {
    pub platform: Platform,
    pub translucent: bool,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
}
