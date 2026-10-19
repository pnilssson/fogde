use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Git is not installed")]
    Missing,

    #[error("{0}")]
    Git(String),

    #[error("could not run Git: {0}")]
    Io(#[from] io::Error),
}
