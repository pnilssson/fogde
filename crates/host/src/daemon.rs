use std::fs::{File, Permissions, TryLockError};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use tokio::fs;
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Notify;

use crate::server;
use crate::shared::{self, Shared};

struct Daemon {
    shared: Arc<Shared>,
    build: String,
    stop: Notify,
}

#[derive(Deserialize)]
struct Client {
    build: String,
}

pub async fn serve(build: String) -> io::Result<()> {
    let path = socket()?;
    let app = path
        .parent()
        .ok_or_else(|| io::Error::other("no app folder"))?;
    fs::create_dir_all(app).await?;
    fs::set_permissions(app, Permissions::from_mode(0o700)).await?;
    let lock = claim(&app.join("host.lock")).await?;
    let _ = fs::remove_file(&path).await;

    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, Permissions::from_mode(0o600)).await?;
    tracing::info!(%build, "host started");

    let daemon = Arc::new(Daemon {
        shared: Arc::new(Shared::new(None)?),
        build,
        stop: Notify::new(),
    });
    let mut terminate = signal(SignalKind::terminate())?;
    let stopping = Arc::clone(&daemon);
    let app = Router::new()
        .route("/", get(upgrade))
        .with_state(Arc::clone(&daemon));

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                () = stopping.stop.notified() => {}
                _ = terminate.recv() => {}
            }
            let _ = fs::remove_file(&path).await;
            drop(lock);
            tracing::info!("host stopping");
        })
        .await?;
    tracing::info!("host stopped");

    Ok(())
}

pub async fn proxy(build: &str) -> io::Result<()> {
    let path = socket()?;
    let mut host = if let Ok(host) = UnixStream::connect(&path).await {
        host
    } else {
        start(build).await?;
        wait(&path).await?
    };
    let mut ssh = tokio::io::join(tokio::io::stdin(), tokio::io::stdout());

    tokio::io::copy_bidirectional(&mut ssh, &mut host).await?;

    Ok(())
}

async fn upgrade(
    upgrade: WebSocketUpgrade,
    Query(client): Query<Client>,
    State(daemon): State<Arc<Daemon>>,
) -> Response {
    if client.build != daemon.build {
        tracing::info!(wanted = %client.build, "stopping for another build");
        daemon.stop.notify_one();
        return StatusCode::CONFLICT.into_response();
    }

    let shared = Arc::clone(&daemon.shared);
    upgrade.on_upgrade(move |socket| server::serve(socket, shared, false))
}

async fn claim(path: &Path) -> io::Result<File> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .await?
        .into_std()
        .await;

    for _ in 0..50 {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(TryLockError::WouldBlock) => tokio::time::sleep(Duration::from_millis(100)).await,
            Err(TryLockError::Error(error)) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AddrInUse,
        "the host is already running",
    ))
}

async fn start(build: &str) -> io::Result<()> {
    let exe = std::env::current_exe()?;
    if lingers().await && service(&exe, build).await {
        return Ok(());
    }

    std::process::Command::new(exe)
        .args(["serve", "--build", build])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;

    Ok(())
}

async fn lingers() -> bool {
    if quiet(
        "loginctl",
        &["--no-ask-password".into(), "enable-linger".into()],
    )
    .await
    {
        return true;
    }

    std::env::var_os("USER")
        .is_some_and(|user| Path::new("/var/lib/systemd/linger").join(user).exists())
}

async fn service(exe: &Path, build: &str) -> bool {
    let mut args: Vec<std::ffi::OsString> = vec![
        "--user".into(),
        "--no-ask-password".into(),
        "--collect".into(),
        "--quiet".into(),
    ];
    for name in ["HOME", "PATH"] {
        if let Some(value) = std::env::var_os(name) {
            let mut setting = std::ffi::OsString::from(format!("--setenv={name}="));
            setting.push(value);
            args.push(setting);
        }
    }
    args.extend([
        "--".into(),
        exe.into(),
        "serve".into(),
        "--build".into(),
        build.into(),
    ]);

    quiet("systemd-run", &args).await
}

async fn quiet(program: &str, args: &[std::ffi::OsString]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .is_ok_and(|status| status.success())
}

async fn wait(path: &Path) -> io::Result<UnixStream> {
    for _ in 0..50 {
        if let Ok(host) = UnixStream::connect(path).await {
            return Ok(host);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    UnixStream::connect(path).await
}

pub fn socket() -> io::Result<PathBuf> {
    Ok(shared::fogde()?.join(".app").join("host.sock"))
}
