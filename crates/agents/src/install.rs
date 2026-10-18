use std::env;
use std::path::PathBuf;

use tokio::process::Command;

use crate::Agent;

#[cfg(windows)]
pub fn dirs(agent: Agent) -> Vec<PathBuf> {
    let home = env::home_dir();
    let local = env::var_os("LOCALAPPDATA").map(PathBuf::from);

    match agent {
        Agent::Claude => home.map(|home| home.join(".local").join("bin")),
        Agent::Codex => local.map(|local| {
            local
                .join("Programs")
                .join("OpenAI")
                .join("Codex")
                .join("bin")
        }),
        Agent::Copilot => {
            return local
                .map(|local| winget(&local.join("Microsoft").join("WinGet"), "GitHub.Copilot_"))
                .unwrap_or_default();
        }
    }
    .into_iter()
    .collect()
}

#[cfg(windows)]
fn winget(root: &std::path::Path, package: &str) -> Vec<PathBuf> {
    let packages = std::fs::read_dir(root.join("Packages"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(package))
        .map(|entry| entry.path());

    std::iter::once(root.join("Links"))
        .chain(packages)
        .collect()
}

#[cfg(not(windows))]
pub fn dirs(_agent: Agent) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = env::home_dir()
        .map(|home| home.join(".local").join("bin"))
        .into_iter()
        .collect();
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));

    dirs
}

#[cfg(windows)]
pub fn installer(agent: Agent) -> Command {
    match agent {
        Agent::Claude => powershell("irm https://claude.ai/install.ps1 | iex"),
        Agent::Codex => {
            let mut command = powershell("irm https://chatgpt.com/codex/install.ps1 | iex");
            command.env("CODEX_NON_INTERACTIVE", "1");
            command
        }
        Agent::Copilot => {
            let mut command = process::command("winget");
            command.args([
                "install",
                "--id",
                "GitHub.Copilot",
                "--exact",
                "--silent",
                "--disable-interactivity",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]);
            command
        }
    }
}

#[cfg(windows)]
fn powershell(script: &str) -> Command {
    let mut command = process::command("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    command
}

#[cfg(not(windows))]
pub fn installer(agent: Agent) -> Command {
    match agent {
        Agent::Claude => sh("curl -fsSL https://claude.ai/install.sh | bash"),
        Agent::Codex => {
            let mut command = sh("curl -fsSL https://chatgpt.com/codex/install.sh | sh");
            command.env("CODEX_NON_INTERACTIVE", "1");
            command
        }
        Agent::Copilot => {
            let mut command = sh("curl -fsSL https://gh.io/copilot-install | bash");
            if let Some(home) = env::home_dir() {
                let local = home.join(".local");
                let path = env::join_paths(
                    std::iter::once(local.join("bin"))
                        .chain(env::var_os("PATH").iter().flat_map(env::split_paths)),
                );
                command.env("PREFIX", &local);
                if let Ok(path) = path {
                    command.env("PATH", path);
                }
            }
            command
        }
    }
}

#[cfg(not(windows))]
fn sh(script: &str) -> Command {
    let mut command = process::command("/bin/sh");
    command.args(["-c", script]);
    command
}
