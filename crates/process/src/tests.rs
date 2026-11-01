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
async fn ends_the_whole_tree_when_dropped() {
    let marker = "127.0.0.78";
    let query = format!(
        "$found = Get-CimInstance Win32_Process -Filter \"Name = 'ping.exe' and CommandLine like '%{marker}%'\" | Select-Object -First 1; if (-not $found) {{ 'missing' }} elseif (Get-Process -Id $found.ParentProcessId -ErrorAction SilentlyContinue) {{ 'parented' }} else {{ 'orphaned' }}"
    );
    let state = || {
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &query])
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    };

    let mut ping = parentless("ping", &["-n", "30", marker]);
    ping.stdout(Stdio::null());
    let child = tree(ping).unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let running = state();
    drop(child);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let ended = state();

    assert_eq!((running.as_str(), ended.as_str()), ("orphaned", "missing"));
}

#[cfg(windows)]
#[tokio::test]
async fn keeps_its_pipes_without_a_parent() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut echo = parentless("findstr", &["x"]);
    echo.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = tree(echo).unwrap();
    let mut stdin = child.stdin().unwrap();
    stdin.write_all(b"xhello\r\n").await.unwrap();
    drop(stdin);
    let mut echoed = String::new();
    child
        .stdout()
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
fn leaves_the_terminal_it_was_started_in() {
    let pane = vars(&[
        ("HC_SESSION", "fogde-1a2b3c4d5e6f"),
        ("HC_TERMINAL_ID", "t3"),
        (
            "HC_TERMINAL_SOCKET",
            "/run/user/1000/herdr-core/fogde-1a2b3c4d5e6f.sock",
        ),
        ("XDG_CONFIG_HOME", ""),
        ("XDG_STATE_HOME", ""),
        ("XDG_DATA_HOME", "/home/dev/.local/share"),
        ("PATH", "/usr/bin"),
    ]);

    assert_eq!(
        leaked(pane),
        [
            "HC_SESSION",
            "HC_TERMINAL_ID",
            "HC_TERMINAL_SOCKET",
            "XDG_CONFIG_HOME",
            "XDG_STATE_HOME",
        ]
    );
}
