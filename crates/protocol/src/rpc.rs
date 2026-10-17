use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PARSE_ERROR: i32 = -32700;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const SERVER_ERROR: i32 = -32000;

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
    fn reads_request_without_params() {
        let request: Request =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":3,"method":"github.user"}"#).unwrap();

        assert_eq!((request.id, request.method.as_str()), (3, "github.user"));
        assert!(request.params.is_null());
    }
}
