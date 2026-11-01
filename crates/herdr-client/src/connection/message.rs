use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;

pub(super) fn answer(line: &str) -> Result<(Option<u64>, Result<Value, Error>), Error> {
    let reply: Reply = serde_json::from_str(line)?;
    let result = match (reply.result, reply.error) {
        (_, Some(failure)) => Err(Error::Herdr {
            kind: failure.data.and_then(|data| data.kind).unwrap_or_default(),
            message: failure.message,
        }),
        (Some(result), None) => Ok(result),
        (None, None) => Ok(Value::Null),
    };

    Ok((reply.id, result))
}

pub(super) fn reason(payload: &[u8]) -> Option<String> {
    if payload.is_empty() {
        return None;
    }

    let closed: Option<Closed> = serde_json::from_slice(payload).ok();

    Some(closed.map_or_else(
        || String::from_utf8_lossy(payload).into_owned(),
        |closed| closed.reason,
    ))
}

#[derive(Serialize)]
pub(super) struct Request<'a, P> {
    pub(super) jsonrpc: &'a str,
    pub(super) id: u64,
    pub(super) method: &'a str,
    pub(super) params: P,
}

#[derive(Serialize)]
pub(super) struct Hello {
    pub(super) generation: u32,
    pub(super) framed: bool,
}

#[derive(Deserialize)]
pub(super) struct Greeting {
    pub(super) version: String,
}

#[derive(Deserialize)]
pub(super) struct Opened {
    pub(super) channel: u32,
}

#[derive(Deserialize)]
struct Reply {
    pub(super) id: Option<u64>,
    pub(super) result: Option<Value>,
    pub(super) error: Option<Failure>,
}

#[derive(Deserialize)]
struct Failure {
    pub(super) message: String,
    pub(super) data: Option<FailureData>,
}

#[derive(Deserialize)]
struct FailureData {
    pub(super) kind: Option<String>,
}

#[derive(Deserialize)]
struct Closed {
    pub(super) reason: String,
}
