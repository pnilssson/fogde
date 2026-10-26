use std::path::Path;

use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::shell::{GetShell, ListShells, SetShell, Shell, ShellOption};

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetShell>();
    router.add::<SetShell>();
    router.add::<ListShells>();
}

impl Handler for GetShell {
    async fn handle(session: &Session, (): ()) -> Result<Shell, Failure> {
        Ok(Shell {
            program: session.shared.settings.shell().await.map_err(failure)?,
        })
    }
}

impl Handler for ListShells {
    async fn handle(_: &Session, (): ()) -> Result<Vec<ShellOption>, Failure> {
        Ok(installed().await)
    }
}

impl Handler for SetShell {
    async fn handle(session: &Session, shell: Shell) -> Result<(), Failure> {
        if let Some(program) = &shell.program
            && !installed()
                .await
                .iter()
                .any(|option| &option.program == program)
        {
            return Err(Failure::new(SERVER_ERROR, "that shell is not installed"));
        }

        session
            .shared
            .settings
            .set_shell(shell.program.clone())
            .await
            .map_err(failure)?;

        session
            .herdr()
            .await?
            .set_shell(shell.program.as_deref())
            .await
            .map_err(failure)
    }
}

async fn installed() -> Vec<ShellOption> {
    tokio::task::spawn_blocking(find).await.unwrap_or_default()
}

pub async fn name(program: Option<String>) -> String {
    tokio::task::spawn_blocking(move || {
        let program = program.unwrap_or_else(default);

        named(&program, &find())
    })
    .await
    .unwrap_or_else(|_| "Terminal".to_owned())
}

fn named(program: &str, options: &[ShellOption]) -> String {
    let file = |program: &str| {
        Path::new(program)
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
    };
    let wanted = file(program);

    options
        .iter()
        .find(|option| file(&option.program) == wanted)
        .map_or_else(
            || {
                Path::new(program).file_stem().map_or_else(
                    || program.to_owned(),
                    |stem| stem.to_string_lossy().into_owned(),
                )
            },
            |option| option.name.clone(),
        )
}

#[cfg(windows)]
fn default() -> String {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let pwsh = std::env::split_paths(&path)
        .filter(|dir| dir.is_absolute())
        .any(|dir| dir.join("pwsh.exe").is_file());

    if pwsh { "pwsh.exe" } else { "powershell.exe" }.to_owned()
}

#[cfg(unix)]
fn default() -> String {
    std::env::var("SHELL")
        .ok()
        .map(|shell| shell.trim().to_owned())
        .filter(|shell| !shell.is_empty())
        .unwrap_or_else(|| "/bin/sh".to_owned())
}

#[cfg(windows)]
fn find() -> Vec<ShellOption> {
    use std::path::PathBuf;

    const KNOWN: [(&str, &str); 4] = [
        ("PowerShell", "pwsh.exe"),
        ("Windows PowerShell", "powershell.exe"),
        ("Command Prompt", "cmd.exe"),
        ("Nushell", "nu.exe"),
    ];

    let path = std::env::var_os("PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    let git = std::env::var_os("ProgramFiles").map(|files| {
        PathBuf::from(files)
            .join("Git")
            .join("bin")
            .join("bash.exe")
    });

    KNOWN
        .iter()
        .filter_map(|&(name, exe)| {
            let program = dirs
                .iter()
                .map(|dir| dir.join(exe))
                .find(|found| found.is_file())?;

            Some((name, program))
        })
        .chain(
            git.filter(|bash| bash.is_file())
                .map(|bash| ("Git Bash", bash)),
        )
        .map(|(name, program)| ShellOption {
            name: name.to_owned(),
            program: program.display().to_string(),
        })
        .collect()
}

#[cfg(unix)]
fn find() -> Vec<ShellOption> {
    let listed = std::fs::read_to_string("/etc/shells").unwrap_or_default();

    parse(&listed)
}

#[cfg(unix)]
fn parse(listed: &str) -> Vec<ShellOption> {
    use std::path::PathBuf;

    let mut shells: Vec<ShellOption> = Vec::new();
    for line in listed.lines().map(str::trim) {
        let program = PathBuf::from(line);
        let Some(name) = program.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !program.is_absolute()
            || !program.is_file()
            || matches!(name, "nologin" | "false")
            || shells.iter().any(|shell| shell.name == name)
        {
            continue;
        }

        shells.push(ShellOption {
            name: name.to_owned(),
            program: line.to_owned(),
        });
    }

    shells
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn lists_each_installed_shell_once() {
        let listed = "# shells\n/bin/sh\n/usr/bin/sh\n/missing/zsh\nbash\n\n";

        let shells = parse(listed);

        assert_eq!(shells.len(), 1);
        assert_eq!(shells[0].name, "sh");
        assert_eq!(shells[0].program, "/bin/sh");
    }
}
