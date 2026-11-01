use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not start Herdr: {0}")]
    Start(#[source] io::Error),

    #[error("lost the connection to Herdr")]
    Io(#[from] io::Error),

    #[error("Herdr closed the connection")]
    Closed,

    #[error("Herdr did not answer in time")]
    Timeout,

    #[error("Herdr sent a message that could not be read")]
    Json(#[from] serde_json::Error),

    #[error("Herdr rejected the request: {message} ({kind})")]
    Herdr { kind: String, message: String },

    #[error("Herdr is {running}, not {wanted}")]
    Version { running: String, wanted: String },

    #[error("Herdr is not available for this platform")]
    Unsupported,

    #[error("could not set up Herdr: {0}")]
    Install(#[from] download::Error),

    #[error("the terminal ended: {0}")]
    Ended(String),
}

impl Error {
    pub fn is_gone(&self) -> bool {
        matches!(self, Self::Herdr { kind, .. } if kind == "not_found")
    }
}
