mod feed;

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

pub use feed::feed;

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

fn scrubbed(mut command: Command) -> Command {
    for name in leaked(env::vars_os()) {
        command.env_remove(name);
    }

    #[cfg(unix)]
    command.process_group(0);

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

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

    #[cfg(windows)]
    #[tokio::test]
    async fn leaves_ctrl_c_working_for_its_children() {
        let marker = "127.0.0.79";
        let script = env::temp_dir().join(format!("process-ctrl-c-{}.ps1", std::process::id()));
        std::fs::write(
            &script,
            format!(
                "Add-Type -Namespace W -Name K -MemberDefinition '[DllImport(\"kernel32.dll\")] public static extern bool GenerateConsoleCtrlEvent(uint e, uint g);'\n\
                 Start-Process ping -ArgumentList '-t','{marker}' -NoNewWindow\n\
                 Start-Sleep 1\n\
                 [W.K]::GenerateConsoleCtrlEvent(0, 0)\n\
                 Start-Sleep 5\n"
            ),
        )
        .unwrap();

        detached(
            "powershell",
            &["-NoProfile", "-File", script.to_str().unwrap()],
        )
        .spawn()
        .unwrap()
        .wait()
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;

        let query = format!(
            "Get-CimInstance Win32_Process -Filter \"Name = 'ping.exe' and CommandLine like '%{marker}%'\" | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force; 'running' }}"
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &query])
            .output()
            .unwrap();
        std::fs::remove_file(&script).unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn keeps_its_pipes_without_a_parent() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut echo = parentless("findstr", &["x"]);
        echo.stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = echo.spawn().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(b"xhello\r\n").await.unwrap();
        drop(stdin);
        let mut echoed = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut echoed)
            .await
            .unwrap();

        assert_eq!(echoed.trim(), "xhello");
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
