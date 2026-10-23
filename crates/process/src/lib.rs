use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

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
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

pub fn detached(program: impl AsRef<OsStr>, args: &[&str]) -> Command {
    let mut command = orphaned(program.as_ref(), args);
    for name in leaked(env::vars_os()) {
        command.env_remove(name);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    command.process_group(0);

    #[cfg(windows)]
    {
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }

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
            name.starts_with("HERDR_") || (name.starts_with("XDG_") && value.is_empty())
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
mod tests {
    use std::ffi::OsString;

    use super::*;

    #[cfg(windows)]
    #[tokio::test]
    async fn leaves_no_parent_behind() {
        let marker = "127.0.0.77";
        detached("ping", &["-n", "4", marker])
            .spawn()
            .unwrap()
            .wait()
            .await
            .unwrap();

        let query = format!(
            "$found = Get-CimInstance Win32_Process -Filter \"Name = 'ping.exe' and CommandLine like '%{marker}%'\" | Select-Object -First 1; if (-not $found) {{ 'missing' }} elseif (Get-Process -Id $found.ParentProcessId -ErrorAction SilentlyContinue) {{ 'parented' }} else {{ 'orphaned' }}"
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &query])
            .output()
            .unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "orphaned");
    }

    fn vars(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
        pairs
            .iter()
            .map(|(name, value)| (OsString::from(name), OsString::from(value)))
            .collect()
    }

    #[test]
    fn leaves_the_pane_it_was_started_in() {
        let pane = vars(&[
            ("HERDR_ENV", "1"),
            ("HERDR_PANE_ID", "w1:p1"),
            ("HERDR_SOCKET_PATH", "/home/dev/.config/herdr/herdr.sock"),
            ("XDG_CONFIG_HOME", ""),
            ("XDG_STATE_HOME", ""),
            ("XDG_DATA_HOME", "/home/dev/.local/share"),
            ("PATH", "/usr/bin"),
        ]);

        assert_eq!(
            leaked(pane),
            [
                "HERDR_ENV",
                "HERDR_PANE_ID",
                "HERDR_SOCKET_PATH",
                "XDG_CONFIG_HOME",
                "XDG_STATE_HOME",
            ]
        );
    }
}
