use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the GitHub CLI is not installed")]
    Missing,

    #[error("the GitHub CLI is not available for this platform")]
    Unsupported,

    #[error("could not set up the GitHub CLI: {0}")]
    Install(String),

    #[error("the downloaded GitHub CLI did not match its checksum")]
    Checksum,

    #[error("could not run the GitHub CLI")]
    Io(#[from] io::Error),

    #[error("the GitHub CLI failed: {0}")]
    Gh(String),

    #[error("the GitHub CLI sent output that could not be read")]
    Json(#[from] serde_json::Error),
}
