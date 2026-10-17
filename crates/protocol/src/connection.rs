use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
pub struct Connection {
    pub url: String,
    pub token: String,
}
