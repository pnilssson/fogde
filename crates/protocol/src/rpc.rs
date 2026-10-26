use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Event;

pub const PARSE_ERROR: i32 = -32700;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const SERVER_ERROR: i32 = -32000;
pub const OFFLINE: i32 = -32001;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct Response {
    jsonrpc: &'static str,
    id: Option<u64>,
    #[serde(flatten)]
    outcome: Outcome,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Outcome {
    Result(Value),
    Error(Failure),
}

#[derive(Debug, Serialize)]
pub struct Notification {
    jsonrpc: &'static str,
    method: &'static str,
    params: Value,
}

impl Notification {
    pub fn new<E: Event>(params: E::Params) -> serde_json::Result<Self> {
        Ok(Self {
            jsonrpc: "2.0",
            method: E::NAME,
            params: serde_json::to_value(params)?,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Message {
    Response(Response),
    Notification(Notification),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Failure {
    pub code: i32,
    pub message: String,
}

impl Failure {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Response {
    pub fn new(id: Option<u64>, outcome: Result<Value, Failure>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            outcome: match outcome {
                Ok(value) => Outcome::Result(value),
                Err(failure) => Outcome::Error(failure),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::terminals::{EndedEvent, TerminalEnded};

    #[test]
    fn writes_result_and_error() {
        let result = Response::new(Some(1), Ok(json!({"login": "octocat"})));
        let error = Response::new(None, Err(Failure::new(PARSE_ERROR, "bad json")));

        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({"jsonrpc": "2.0", "id": 1, "result": {"login": "octocat"}})
        );
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "bad json"}})
        );
    }

    #[test]
    fn writes_notification() {
        let notification = Message::Notification(
            Notification::new::<EndedEvent>(TerminalEnded {
                id: "w1".into(),
                error: None,
            })
            .unwrap(),
        );

        assert_eq!(
            serde_json::to_value(notification).unwrap(),
            json!({"jsonrpc": "2.0", "method": "terminals.ended", "params": {"id": "w1", "error": null}})
        );
    }

    #[test]
    fn reads_request_without_params() {
        let request: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":3,"method":"github.user"}"#).unwrap();

        assert_eq!((request.id, request.method.as_str()), (3, "github.user"));
        assert!(request.params.is_null());
    }
}
