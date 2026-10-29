use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the GitHub CLI is not installed")]
    Missing,

    #[error("the GitHub CLI is not available for this platform")]
    Unsupported,

    #[error("could not set up the GitHub CLI: {0}")]
    Install(#[from] download::Error),

    #[error("could not run the GitHub CLI")]
    Io(#[from] io::Error),

    #[error("{0} is not a GitHub address")]
    Host(String),

    #[error("{0} is not a repository name")]
    Repo(String),

    #[error("there is already a folder at {}", .0.display())]
    Exists(PathBuf),

    #[error("{0}")]
    Invalid(String),

    #[error("the GitHub CLI failed: {0}")]
    Gh(String),

    #[error("the GitHub CLI sent output that could not be read")]
    Json(#[from] serde_json::Error),
}
