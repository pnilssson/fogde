use std::io;
use std::path::{Path, PathBuf};
use std::process::Output;

use tokio::process::Command;

use crate::{Error, install};

#[derive(Clone, Debug)]
pub struct Gh {
    program: PathBuf,
}

impl Gh {
    pub async fn locate(tools: &Path) -> Result<Self, Error> {
        Ok(Self {
            program: install::locate(tools).await?,
        })
    }

    pub(crate) fn command(&self) -> Command {
        process::command(&self.program)
    }

    pub(crate) async fn run(&self, args: &[&str]) -> Result<Output, Error> {
        self.command()
            .args(args)
            .output()
            .await
            .map_err(spawn_error)
    }
}

pub fn spawn_error(error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::NotFound {
        return Error::Missing;
    }

    Error::Io(error)
}

pub fn failure(output: &[u8]) -> Error {
    Error::Gh(String::from_utf8_lossy(output).trim().to_owned())
}
