mod feed;

use std::env;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;
use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
#[cfg(windows)]
use process_wrap::tokio::{CreationFlags, JobObject};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
#[cfg(windows)]
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

pub use feed::feed;

pub const UNPROMPTED: [(&str, &str); 4] = [
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GCM_INTERACTIVE", "never"),
    ("GIT_ASKPASS", ""),
    ("SSH_ASKPASS_REQUIRE", "never"),
];

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = hidden(program);
    for name in leaked(env::vars_os()) {
        command.env_remove(name);
    }

    command
}

pub fn hidden(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command.kill_on_drop(true);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW.0);

    command
}

pub fn detached(program: impl AsRef<OsStr>, args: &[&str]) -> Command {
    let mut command = scrubbed(launched(program.as_ref(), args));
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command
}

pub fn parentless(program: impl AsRef<OsStr>, args: &[&str]) -> Command {
    let mut command = scrubbed(orphaned(program.as_ref(), args));
    command.kill_on_drop(true);

    command
}

pub struct Tree {
    child: Box<dyn ChildWrapper>,
    done: bool,
}

impl Tree {
    pub fn stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin().take()
    }

    pub fn stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout().take()
    }

    pub fn stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr().take()
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait().await?;
        self.done = true;

        Ok(status)
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.child.start_kill();
        }
    }
}

pub fn tree(command: Command) -> io::Result<Tree> {
    let mut command = CommandWrap::from(command);
    command.wrap(KillOnDrop);

    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());

    #[cfg(windows)]
    command
        .wrap(CreationFlags(CREATE_NO_WINDOW))
        .wrap(JobObject);

    command.spawn().map(|child| Tree { child, done: false })
}

pub fn not_found<E: From<io::Error>>(missing: E) -> impl FnOnce(io::Error) -> E {
    move |error| {
        if error.kind() == io::ErrorKind::NotFound {
            return missing;
        }

        E::from(error)
    }
}

fn scrubbed(mut command: Command) -> Command {
    for name in leaked(env::vars_os()) {
        command.env_remove(name);
    }

    #[cfg(unix)]
    command.process_group(0);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW.0);

    command
}

#[cfg(windows)]
fn launched(program: &OsStr, args: &[&str]) -> Command {
    let parts: Option<Vec<&str>> = std::iter::once(program.to_str())
        .chain(args.iter().copied().map(Some))
        .map(|part| part.filter(|part| !part.contains('"')))
        .collect();
    let Some(parts) = parts else {
        let mut command = Command::new(program);
        command.args(args);
        return command;
    };

    let literal = |part: &str| format!("'{}'", part.replace('\'', "''"));
    let arguments: Vec<String> = parts[1..]
        .iter()
        .map(|part| format!("([char]34 + {} + [char]34)", literal(part)))
        .collect();
    let list = if arguments.is_empty() {
        String::new()
    } else {
        format!(" -ArgumentList @({})", arguments.join(", "))
    };

    let mut command = Command::new("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!(
            "Add-Type -Namespace Fogde -Name Console -MemberDefinition ('[DllImport(' + [char]34 + 'kernel32.dll' + [char]34 + ')] public static extern bool SetConsoleCtrlHandler(IntPtr handler, bool add);'); [Fogde.Console]::SetConsoleCtrlHandler([IntPtr]::Zero, $false) | Out-Null; Start-Process -FilePath {}{list} -WindowStyle Hidden",
            literal(parts[0])
        ),
    ]);
    command
}

#[cfg(not(windows))]
fn launched(program: &OsStr, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

#[cfg(windows)]
fn orphaned(program: &OsStr, args: &[&str]) -> Command {
    let quoted: Option<Vec<String>> = std::iter::once(program.to_str())
        .chain(args.iter().copied().map(Some))
        .map(|part| {
            part.filter(|part| !part.contains(['"', '%', '!']))
                .map(|part| format!("\"{part}\""))
        })
        .collect();
    let Some(quoted) = quoted else {
        let mut command = Command::new(program);
        command.args(args);
        return command;
    };

    let mut command = Command::new(env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()));
    command.raw_arg(format!("/d /v:off /c start \"\" /b {}", quoted.join(" ")));
    command
}

#[cfg(not(windows))]
fn orphaned(program: &OsStr, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

fn leaked(vars: impl IntoIterator<Item = (OsString, OsString)>) -> Vec<OsString> {
    vars.into_iter()
        .filter(|(name, value)| {
            let name = name.to_string_lossy();
            name.starts_with("HC_") || (name.starts_with("XDG_") && value.is_empty())
        })
        .map(|(name, _)| name)
        .collect()
}

pub fn find(name: &str) -> Option<PathBuf> {
    find_in(env::split_paths(&env::var_os("PATH")?), name)
}

pub fn find_in(dirs: impl IntoIterator<Item = PathBuf>, name: &str) -> Option<PathBuf> {
    let names: Vec<String> = if cfg!(windows) {
        vec![format!("{name}.exe"), format!("{name}.cmd")]
    } else {
        vec![name.to_owned()]
    };

    dirs.into_iter()
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests;
