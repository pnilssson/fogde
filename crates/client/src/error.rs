use protocol::rpc::{Failure, OFFLINE};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Could not connect.")]
    Unreachable,
    #[error("The connection timed out.")]
    Timeout,
    #[error("The connection was replaced.")]
    Replaced,
    #[error("The connection closed.")]
    Closed,
    #[error("The connection was lost.")]
    Lost,
    #[error("The host sent a reply this app cannot read: {0}")]
    Invalid(String),
    #[error("{}", .0.message)]
    Rpc(Failure),
}

impl Error {
    pub fn retryable(&self) -> bool {
        match self {
            Self::Rpc(failure) => failure.code == OFFLINE,
            _ => true,
        }
    }
}
