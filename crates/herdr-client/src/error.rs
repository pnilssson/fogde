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
}
