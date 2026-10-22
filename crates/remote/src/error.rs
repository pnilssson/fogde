use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} is not a server address")]
    Target(String),

    #[error("SSH is not installed")]
    Missing,

    #[error("{0} did not accept your SSH key")]
    Key(String),

    #[error("could not reach {0}")]
    Unreachable(String),

    #[error("the identity of {0} changed since you last connected, so the connection was stopped")]
    Changed(String),

    #[error("{0} runs {1}, and only 64-bit Linux servers are supported")]
    Unsupported(String, String),

    #[error("the server build for {0} is missing from this build of the app")]
    Binary(String),

    #[error("{0} runs a newer version, so update the app to connect")]
    Outdated(String),

    #[error("SSH failed: {0}")]
    Ssh(String),

    #[error("the connection to the server failed: {0}")]
    Tunnel(String),

    #[error(
        "servers need a Fogde folder named with only letters, digits, '-', '_' and '.', not {0}"
    )]
    Folder(String),

    #[error("could not run SSH")]
    Io(#[from] io::Error),
}

impl Error {
    pub fn transient(&self) -> bool {
        matches!(
            self,
            Self::Unreachable(_) | Self::Ssh(_) | Self::Tunnel(_) | Self::Io(_)
        )
    }
}
