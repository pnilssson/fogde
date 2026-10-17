use std::fmt;
use std::process::{Output, Stdio};

use crate::{Error, status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Codex,
}

impl Agent {
    pub async fn signed_in(self) -> Result<bool, Error> {
        let output = match self.run(self.status()).await {
            Ok(output) => output,
            Err(Error::Missing(_)) => return Ok(false),
            Err(error) => return Err(error),
        };

        match self {
            Self::Claude => status::claude(&output.stdout),
            Self::Codex => status::codex(output.status.success(), &output.stderr),
        }
        .ok_or_else(|| self.failure(&output))
    }

    pub async fn sign_in(self) -> Result<(), Error> {
        let output = self.run(self.login()).await?;
        if !output.status.success() {
            return Err(self.failure(&output));
        }

        Ok(())
    }

    fn binary(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }

    fn status(self) -> &'static [&'static str] {
        match self {
            Self::Claude => &["auth", "status", "--json"],
            Self::Codex => &["login", "status"],
        }
    }

    fn login(self) -> &'static [&'static str] {
        match self {
            Self::Claude => &["auth", "login"],
            Self::Codex => &["login"],
        }
    }

    async fn run(self, args: &[&str]) -> Result<Output, Error> {
        let program = process::find(self.binary()).ok_or(Error::Missing(self))?;

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
        })
    }
}
