use std::path::PathBuf;
use std::process::{Output, Stdio};

use protocol::agents::Agent;
use tokio::process::Command;

use crate::install::{self, Install};
use crate::{Error, status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Missing,
    SignedOut,
    SignedIn,
}

struct Cli {
    status: Option<Status>,
    login: Option<Login>,
    paste: bool,
    install: Install,
}

struct Status {
    args: &'static [&'static str],
    signed_in: fn(&Output) -> Option<bool>,
}

struct Login {
    local: &'static [&'static str],
    remote: &'static [&'static str],
}

const CLAUDE: Cli = Cli {
    status: Some(Status {
        args: &["auth", "status", "--json"],
        signed_in: |output| status::claude(&output.stdout),
    }),
    login: Some(Login {
        local: &["auth", "login"],
        remote: &["auth", "login"],
    }),
    paste: true,
    install: install::CLAUDE,
};

const CODEX: Cli = Cli {
    status: Some(Status {
        args: &["login", "status"],
        signed_in: |output| status::codex(output.status.success(), &output.stderr),
    }),
    login: Some(Login {
        local: &["login"],
        remote: &["login", "--device-auth"],
    }),
    paste: false,
    install: install::CODEX,
};

const COPILOT: Cli = Cli {
    status: None,
    login: None,
    paste: false,
    install: install::COPILOT,
};

const fn cli(agent: Agent) -> &'static Cli {
    match agent {
        Agent::Claude => &CLAUDE,
        Agent::Codex => &CODEX,
        Agent::Copilot => &COPILOT,
    }
}

pub async fn state(agent: Agent) -> Result<State, Error> {
    if !installed(agent).await {
        return Ok(State::Missing);
    }
    let Some(status) = &cli(agent).status else {
        return Ok(State::SignedIn);
    };
    let output = match run(agent, status.args).await {
        Ok(output) => output,
        Err(Error::Missing(_)) => return Ok(State::Missing),
        Err(error) => return Err(error),
    };

    let signed_in = (status.signed_in)(&output).ok_or_else(|| failure(agent, &output))?;

    Ok(if signed_in {
        State::SignedIn
    } else {
        State::SignedOut
    })
}

pub async fn install(agent: Agent) -> Result<(), Error> {
    let output = (cli(agent).install.command)()
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|error| Error::Io(agent, error))?;
    if !output.status.success() {
        return Err(failure(agent, &output));
    }
    if !installed(agent).await {
        return Err(Error::Failed(
            agent,
            "the installer finished, but the command was not found".to_owned(),
        ));
    }

    Ok(())
}

pub(crate) fn login(agent: Agent, remote: bool) -> Option<&'static [&'static str]> {
    cli(agent)
        .login
        .as_ref()
        .map(|login| if remote { login.remote } else { login.local })
}

pub(crate) fn pastes(agent: Agent) -> bool {
    cli(agent).paste
}

pub(crate) async fn installed(agent: Agent) -> bool {
    locate(agent).await.is_some()
}

async fn locate(agent: Agent) -> Option<PathBuf> {
    let binary = agent.spec().command;
    let dirs = cli(agent).install.dirs;

    tokio::task::spawn_blocking(move || {
        process::find(binary).or_else(|| process::find_in(dirs(), binary))
    })
    .await
    .ok()
    .flatten()
}

pub(crate) async fn command(agent: Agent) -> Result<Command, Error> {
    let program = locate(agent).await.ok_or(Error::Missing(agent))?;
    let mut command = process::command(program);
    command.env("NO_COLOR", "1");

    Ok(command)
}

pub(crate) async fn parentless(agent: Agent, args: &[&str]) -> Result<Command, Error> {
    let program = locate(agent).await.ok_or(Error::Missing(agent))?;
    let mut command = process::parentless(program, args);
    command.env("NO_COLOR", "1");

    Ok(command)
}

async fn run(agent: Agent, args: &[&str]) -> Result<Output, Error> {
    command(agent)
        .await?
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|error| Error::Io(agent, error))
}

fn failure(agent: Agent, output: &Output) -> Error {
    let message = if output.stderr.trim_ascii().is_empty() {
        &output.stdout
    } else {
        &output.stderr
    };

    Error::Failed(agent, String::from_utf8_lossy(message).trim().to_owned())
}
