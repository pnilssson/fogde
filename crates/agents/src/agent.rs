use std::fmt;
use std::path::PathBuf;
use std::process::{Output, Stdio};

use crate::{Error, install, status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
    Copilot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Missing,
    SignedOut,
    SignedIn,
}

impl Agent {
    pub async fn state(self) -> Result<State, Error> {
        if !self.installed().await {
            return Ok(State::Missing);
        }
        let Some(args) = self.status() else {
            return Ok(State::SignedIn);
        };
        let output = match self.run(args).await {
            Ok(output) => output,
            Err(Error::Missing(_)) => return Ok(State::Missing),
            Err(error) => return Err(error),
        };

        let signed_in = match self {
            Self::Claude => status::claude(&output.stdout),
            Self::Codex => status::codex(output.status.success(), &output.stderr),
            Self::Copilot => None,
        }
        .ok_or_else(|| self.failure(&output))?;

        Ok(if signed_in {
            State::SignedIn
        } else {
            State::SignedOut
        })
    }

    pub async fn sign_in(self) -> Result<(), Error> {
        let Some(args) = self.login() else {
            return if self.installed().await {
                Ok(())
            } else {
                Err(Error::Missing(self))
            };
        };
        let output = self.run(args).await?;
        if !output.status.success() {
            return Err(self.failure(&output));
        }

        Ok(())
    }

    pub async fn install(self) -> Result<(), Error> {
        let output = install::installer(self)
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|error| Error::Io(self, error))?;
        if !output.status.success() {
            return Err(self.failure(&output));
        }
        if !self.installed().await {
            return Err(Error::Failed(
                self,
                "the installer finished, but the command was not found".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn uses_github(self) -> bool {
        self == Self::Copilot
    }

    async fn installed(self) -> bool {
        self.locate().await.is_some()
    }

    /// Searching the disk blocks, so it runs off the async runtime.
    async fn locate(self) -> Option<PathBuf> {
        tokio::task::spawn_blocking(move || {
            process::find(self.binary())
                .or_else(|| process::find_in(install::dirs(self), self.binary()))
        })
        .await
        .ok()
        .flatten()
    }

    fn binary(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
        }
    }

    fn status(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Claude => Some(&["auth", "status", "--json"]),
            Self::Codex => Some(&["login", "status"]),
            Self::Copilot => None,
        }
    }

    fn login(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Claude => Some(&["auth", "login"]),
            Self::Codex => Some(&["login"]),
            Self::Copilot => None,
        }
    }

    async fn run(self, args: &[&str]) -> Result<Output, Error> {
        let program = self.locate().await.ok_or(Error::Missing(self))?;

        process::command(program)
            .args(args)
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|error| Error::Io(self, error))
    }

    fn failure(self, output: &Output) -> Error {
        let message = if output.stderr.trim_ascii().is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };

        Error::Failed(self, String::from_utf8_lossy(message).trim().to_owned())
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Copilot => "GitHub Copilot",
        })
    }
}
