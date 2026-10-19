use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not connect to Herdr")]
    Connect(#[source] io::Error),

    #[error("lost the connection to Herdr")]
    Io(#[from] io::Error),

    #[error("Herdr closed the connection")]
    Closed,

    #[error("Herdr sent a message that could not be read")]
    Json(#[from] serde_json::Error),

    #[error("Herdr answered request {actual} instead of {expected}")]
    Mismatch { expected: String, actual: String },

    #[error("Herdr rejected the request: {message} ({code})")]
    Herdr { code: String, message: String },

    #[error("could not start Herdr")]
    Start(#[source] io::Error),

    #[error("Herdr did not start in time")]
    Timeout,

    #[error("Herdr is not available for this platform")]
    Unsupported,

    #[error("could not set up Herdr: {0}")]
    Install(#[from] download::Error),

    #[error("Herdr stopped the terminal stream: {0}")]
    Stream(String),

    #[error("Herdr sent a terminal frame that could not be decoded")]
    Frame(#[from] base64::DecodeError),
}
