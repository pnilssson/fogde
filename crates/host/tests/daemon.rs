#![cfg(unix)]

use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::io::Join;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{WebSocketStream, client_async};

type Socket = WebSocketStream<Join<ChildStdout, ChildStdin>>;

const PING: &str = r#"{"jsonrpc":"2.0","id":1,"method":"onboarding.done"}"#;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn home(name: &str) -> io::Result<PathBuf> {
    let home = std::env::temp_dir().join(format!("fogde-daemon-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home)?;

    Ok(home)
}

fn proxy(home: &Path, build: &str) -> io::Result<Child> {
    Command::new(env!("CARGO_BIN_EXE_fogde-host"))
        .args(["proxy", "--build", build])
        .env("HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

async fn open(child: &mut Child, build: &str, version: &str) -> Result<(Socket, String), Error> {
    let (Some(stdout), Some(stdin)) = (child.stdout.take(), child.stdin.take()) else {
        return Err(Error::ConnectionClosed);
    };
    let url = format!("ws://fogde/?build={build}&version={version}");
    let (socket, response) = client_async(url, tokio::io::join(stdout, stdin)).await?;
    let host = response
        .headers()
        .get("fogde-host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();

    Ok((socket, host))
}

async fn ask(socket: &mut Socket, request: &str) -> Result<Value, Error> {
    socket.send(Message::text(request)).await?;
    let reply = socket.next().await.ok_or(Error::ConnectionClosed)??;

    serde_json::from_str(reply.to_text()?).map_err(|_| Error::ConnectionClosed)
}

async fn connect(home: &Path, build: &str, version: &str) -> Result<(Value, String), Error> {
    let mut child = proxy(home, build).map_err(Error::Io)?;
    let (mut socket, host) = open(&mut child, build, version).await?;

    Ok((ask(&mut socket, PING).await?, host))
}

async fn call(home: &Path, build: &str, request: &str) -> Result<Value, Error> {
    let mut child = proxy(home, build).map_err(Error::Io)?;
    let (mut socket, _) = open(&mut child, build, VERSION).await?;

    ask(&mut socket, request).await
}

async fn switch(home: &Path, build: &str) -> Result<(Value, String), Error> {
    for _ in 0..20 {
        match connect(home, build, VERSION).await {
            Err(Error::Http(response)) if response.status() == 409 => {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            reply => return reply,
        }
    }

    Err(Error::ConnectionClosed)
}

async fn stop(home: &Path) {
    let _ = call(home, "stop", PING).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = std::fs::remove_dir_all(home);
}

#[tokio::test]
async fn starts_the_host_and_stops_it_for_another_build() {
    let home = home("switch").unwrap();

    let reply = call(&home, "one", PING).await.unwrap();
    assert_eq!(reply["result"], json!(false));

    let refused = call(&home, "two", PING).await;
    assert!(matches!(refused, Err(Error::Http(response)) if response.status() == 409));

    stop(&home).await;
}

#[tokio::test]
async fn refuses_an_older_app_and_keeps_running() {
    let home = home("older").unwrap();
    let mut first = proxy(&home, "one").unwrap();
    let (mut kept, _) = open(&mut first, "one", VERSION).await.unwrap();

    let refused = connect(&home, "old", "0.0.0").await;
    assert!(matches!(refused, Err(Error::Http(response)) if response.status() == 426));

    assert_eq!(ask(&mut kept, PING).await.unwrap()["result"], json!(false));

    stop(&home).await;
}

#[tokio::test]
async fn keeps_its_identity_across_builds() {
    let home = home("identity").unwrap();

    let (_, before) = connect(&home, "one", VERSION).await.unwrap();
    let (_, after) = switch(&home, "two").await.unwrap();

    assert_eq!(before.len(), 16);
    assert_eq!(before, after);

    stop(&home).await;
}

#[tokio::test]
async fn removes_other_builds_when_it_starts() {
    let home = home("prune").unwrap();
    let bin = home.join("Fogde/.app/bin");
    std::fs::create_dir_all(&bin).unwrap();
    for name in ["fogde-host-old", "fogde-host-one", "fogde-host-new.partial"] {
        std::fs::write(bin.join(name), "").unwrap();
    }

    call(&home, "one", PING).await.unwrap();

    assert!(!bin.join("fogde-host-old").exists());
    assert!(bin.join("fogde-host-one").exists());
    assert!(bin.join("fogde-host-new.partial").exists());

    stop(&home).await;
}

#[tokio::test]
async fn keeps_the_new_host_reachable_after_a_build_switch() {
    let home = home("handover").unwrap();
    let mut first = proxy(&home, "one").unwrap();
    let (mut old, _) = open(&mut first, "one", VERSION).await.unwrap();
    ask(&mut old, PING).await.unwrap();

    switch(&home, "two").await.unwrap();
    drop(old);
    drop(first);
    tokio::time::sleep(Duration::from_secs(1)).await;

    assert!(home.join("Fogde/.app/host.sock").exists());
    assert_eq!(
        call(&home, "two", PING).await.unwrap()["result"],
        json!(false)
    );

    stop(&home).await;
}

#[tokio::test]
async fn finishes_requests_after_the_client_leaves() {
    let home = home("leaves").unwrap();
    let repos = home.join("chosen").display().to_string();
    let mut child = proxy(&home, "one").unwrap();
    let (mut socket, _) = open(&mut child, "one", VERSION).await.unwrap();

    let set = json!({"jsonrpc": "2.0", "id": 1, "method": "repos.setFolder", "params": repos});
    socket.send(Message::text(set.to_string())).await.unwrap();
    drop(socket);
    child.wait().await.unwrap();

    let get = r#"{"jsonrpc":"2.0","id":2,"method":"repos.folder"}"#;
    let mut folder = Value::Null;
    for _ in 0..50 {
        folder = call(&home, "one", get).await.unwrap()["result"].clone();
        if folder == json!(repos) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(folder, json!(repos));

    stop(&home).await;
}

#[cfg(target_os = "linux")]
fn hosts(home: &Path) -> usize {
    let wanted = format!("HOME={}", home.display());
    let Ok(processes) = std::fs::read_dir("/proc") else {
        return 0;
    };

    processes
        .flatten()
        .filter(|process| {
            let dir = process.path();
            let args = std::fs::read(dir.join("cmdline")).unwrap_or_default();
            let env = std::fs::read(dir.join("environ")).unwrap_or_default();
            args.split(|byte| *byte == 0).any(|arg| arg == b"serve")
                && env
                    .split(|byte| *byte == 0)
                    .any(|var| var == wanted.as_bytes())
        })
        .count()
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn runs_one_host_when_clients_start_it_together() {
    let home = home("together").unwrap();

    let replies = futures_util::future::join_all((0..8).map(|_| call(&home, "one", PING))).await;
    for reply in replies {
        assert_eq!(reply.unwrap()["result"], json!(false));
    }
    tokio::time::sleep(Duration::from_secs(6)).await;

    assert_eq!(hosts(&home), 1);

    stop(&home).await;
}
