use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use serde::de::IgnoredAny;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::{Mutex, oneshot};

use crate::install::{self, VERSION};
use crate::terminal::Empty;
use crate::{Connection, Error};

const ATTEMPTS: usize = 50;
const PAUSE: Duration = Duration::from_millis(100);

pub struct Herdr {
    program: PathBuf,
    env: Vec<(&'static str, PathBuf)>,
    connection: Mutex<Option<Arc<Connection>>>,
}

impl Herdr {
    pub async fn start(tools: &Path, home: &Path) -> Result<Self, Error> {
        let herdr = Self {
            program: install::locate(tools).await?,
            env: env(home),
            connection: Mutex::new(None),
        };

        let running = herdr.client().await?.version().to_owned();
        if running == VERSION {
            return Ok(herdr);
        }

        tracing::info!(%running, "restarting Herdr on {VERSION}");
        let restarted = herdr
            .client()
            .await?
            .call::<IgnoredAny>("server.restart", Empty {})
            .await;
        if let Err(error) = restarted {
            tracing::debug!(%error, "Herdr closed the connection while restarting");
        }
        herdr.forget().await;

        for _ in 0..ATTEMPTS {
            tokio::time::sleep(PAUSE).await;
            if let Ok(connection) = herdr.client().await {
                if connection.version() == VERSION {
                    return Ok(herdr);
                }
                herdr.forget().await;
            }
        }

        Err(Error::Version {
            running,
            wanted: VERSION.to_owned(),
        })
    }

    pub async fn client(&self) -> Result<Arc<Connection>, Error> {
        let mut current = self.connection.lock().await;
        if let Some(connection) = current
            .as_ref()
            .filter(|connection| !connection.is_closed())
        {
            return Ok(Arc::clone(connection));
        }

        let connection = Arc::new(self.connect().await?);
        *current = Some(Arc::clone(&connection));

        Ok(connection)
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    async fn forget(&self) {
        *self.connection.lock().await = None;
    }

    async fn connect(&self) -> Result<Connection, Error> {
        let mut child = process::command(&self.program)
            .arg("connect")
            .envs(self.env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Error::Start)?;
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            return Err(Error::Start(io::Error::other("missing pipe")));
        };
        let (said, last) = oneshot::channel();
        tokio::spawn(log(stderr, said));

        match Connection::open(stdout, stdin, Some(child)).await {
            Err(Error::Closed) => {
                let line = tokio::time::timeout(PAUSE * 10, last).await;
                match line {
                    Ok(Ok(Some(line))) => Err(Error::Start(io::Error::other(line))),
                    _ => Err(Error::Closed),
                }
            }
            opened => opened,
        }
    }
}

fn env(home: &Path) -> Vec<(&'static str, PathBuf)> {
    let session = session(home);

    vec![
        ("HC_SOCKET", socket(home, &session)),
        ("HC_SESSION", PathBuf::from(session)),
    ]
}

#[cfg(unix)]
fn socket(home: &Path, session: &str) -> PathBuf {
    home.join(format!("{session}.sock"))
}

#[cfg(windows)]
fn socket(_home: &Path, session: &str) -> PathBuf {
    let user: String = std::env::var("USERNAME")
        .unwrap_or_default()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();

    PathBuf::from(format!(r"\\.\pipe\herdr-core-{user}-{session}.sock"))
}

async fn log(stderr: impl AsyncRead + Unpin, said: oneshot::Sender<Option<String>>) {
    let mut lines = BufReader::new(stderr).lines();
    let mut last = None;
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!(%line, "Herdr");
        if !line.trim().is_empty() {
            last = Some(line);
        }
    }

    let _ = said.send(last);
}

fn session(home: &Path) -> String {
    let digest = Sha256::digest(home.to_string_lossy().as_bytes());

    format!("fogde-{}", &hex::encode(digest)[..12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gives_each_home_its_own_session() {
        let home = session(Path::new("/home/dev/Fogde/.app/herdr"));
        let dev = session(Path::new("/home/dev/Fogde-dev/.app/herdr"));

        assert_ne!(home, dev);
        assert_eq!(home, session(Path::new("/home/dev/Fogde/.app/herdr")));
        assert!(home.starts_with("fogde-") && home.len() == 18);
    }

    #[cfg(windows)]
    #[test]
    fn names_the_pipe_herdr_would_pick_for_the_session() {
        let home = Path::new(r"C:\Users\dev\Fogde\.app\herdr-core");
        let session = session(home);
        let pipe = socket(home, &session);
        let pipe = pipe.to_string_lossy();

        assert!(pipe.starts_with(r"\\.\pipe\herdr-core-"), "{pipe}");
        assert!(pipe.ends_with(&format!("-{session}.sock")), "{pipe}");
    }

    #[cfg(unix)]
    #[test]
    fn keeps_the_longest_socket_within_what_macos_allows() {
        let home = Path::new("/Users/firstname.lastname/Fogde/.app/herdr-core");
        let holder = socket(home, &session(home)).with_extension("holder.sock");

        assert!(holder.as_os_str().len() + 9 <= 103, "{}", holder.display());
    }
}
