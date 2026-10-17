use std::io;

use crate::Agent;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} is not installed")]
    Missing(Agent),

    #[error("could not run {0}")]
    Io(Agent, #[source] io::Error),

    #[error("{0} failed: {1}")]
    Failed(Agent, String),
}
