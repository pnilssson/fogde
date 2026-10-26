use std::env;
use std::path::PathBuf;

use tokio::process::Command;

pub use os::{CLAUDE, CODEX, COPILOT};

#[derive(Clone, Copy)]
pub struct Install {
    pub command: fn() -> Command,
    pub dirs: fn() -> Vec<PathBuf>,
}

fn local_bin() -> Option<PathBuf> {
    env::home_dir().map(|home| home.join(".local").join("bin"))
}

#[cfg(windows)]
mod os {
    use std::env;
    use std::path::{Path, PathBuf};

    use tokio::process::Command;

    use super::{Install, local_bin};

    pub const CLAUDE: Install = Install {
        command: || powershell("irm https://claude.ai/install.ps1 | iex"),
        dirs: || local_bin().into_iter().collect(),
    };

    pub const CODEX: Install = Install {
        command: || {
            let mut command = powershell("irm https://chatgpt.com/codex/install.ps1 | iex");
            command.env("CODEX_NON_INTERACTIVE", "1");
            command
        },
        dirs: || {
            app_data()
                .map(|local| {
                    local
                        .join("Programs")
                        .join("OpenAI")
                        .join("Codex")
                        .join("bin")
                })
                .into_iter()
                .collect()
        },
    };

    pub const COPILOT: Install = Install {
        command: || {
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
        },
        dirs: || {
            app_data()
                .map(|local| winget(&local.join("Microsoft").join("WinGet"), "GitHub.Copilot_"))
                .unwrap_or_default()
        },
    };

    fn app_data() -> Option<PathBuf> {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    }

    fn winget(root: &Path, package: &str) -> Vec<PathBuf> {
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
}

#[cfg(not(windows))]
mod os {
    use std::env;
    use std::path::PathBuf;

    use tokio::process::Command;

    use super::{Install, local_bin};

    pub const CLAUDE: Install = Install {
        command: || sh("curl -fsSL https://claude.ai/install.sh | bash"),
        dirs,
    };

    pub const CODEX: Install = Install {
        command: || {
            let mut command = sh("curl -fsSL https://chatgpt.com/codex/install.sh | sh");
            command.env("CODEX_NON_INTERACTIVE", "1");
            command
        },
        dirs,
    };

    pub const COPILOT: Install = Install {
        command: || {
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
        },
        dirs,
    };

    fn dirs() -> Vec<PathBuf> {
        local_bin()
            .into_iter()
            .chain(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from))
            .collect()
    }

    fn sh(script: &str) -> Command {
        let mut command = process::command("/bin/sh");
        command.args(["-c", script]);
        command
    }
}
