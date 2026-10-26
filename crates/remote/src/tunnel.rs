use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Join};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::{self, http::StatusCode};

use crate::ssh::{Failure, Server, spawn_error};
use crate::{Error, System};

pub struct Binaries {
    source: Source,
    builds: Mutex<HashMap<String, String>>,
}

enum Source {
    Built(PathBuf),
    Bundled(PathBuf),
}

impl Binaries {
    pub fn built(target: PathBuf) -> Self {
        Self::new(Source::Built(target))
    }

    pub fn bundled(dir: PathBuf) -> Self {
        Self::new(Source::Bundled(dir))
    }

    fn new(source: Source) -> Self {
        Self {
            source,
            builds: Mutex::default(),
        }
    }

    fn path(&self, triple: &str) -> PathBuf {
        match &self.source {
            Source::Built(target) => target.join(triple).join("release").join("fogde-host"),
            Source::Bundled(dir) => dir.join(triple).join("fogde-host"),
        }
    }

    async fn read(&self, triple: &str) -> Result<Vec<u8>, Error> {
        fs::read(self.path(triple))
            .await
            .map_err(|_| Error::Binary(triple.to_owned()))
    }

    async fn build(&self, triple: &str) -> Result<String, Error> {
        let mut builds = self.builds.lock().await;
        if let Some(build) = builds.get(triple) {
            return Ok(build.clone());
        }

        let binary = self.read(triple).await?;
        let build = tokio::task::spawn_blocking(move || build(&binary))
            .await
            .map_err(|_| Error::Binary(triple.to_owned()))?;
        builds.insert(triple.to_owned(), build.clone());

        Ok(build)
    }
}

pub struct Tunnel {
    pub socket: WebSocketStream<Join<ChildStdout, ChildStdin>>,
    pub host: String,
    ssh: Child,
}

impl Tunnel {
    pub fn alive(&mut self) -> bool {
        matches!(self.ssh.try_wait(), Ok(None))
    }
}

enum Attempt {
    NotInstalled,
    Restarting,
    Failed(Error),
}

pub const HOST_HEADER: &str = "fogde-host";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Server {
    pub async fn open(
        &self,
        system: &System,
        binaries: &Binaries,
        home: &Path,
    ) -> Result<Tunnel, Error> {
        let folder = folder(home)?;
        let build = binaries.build(&system.triple).await?;

        let attempt = match self.connect(&build, folder).await {
            Err(Attempt::NotInstalled) => {
                self.install(&binaries.read(&system.triple).await?, &build, folder)
                    .await?;
                self.connect(&build, folder).await
            }
            attempt => attempt,
        };

        attempt.map_err(|attempt| match attempt {
            Attempt::Failed(error) => error,
            Attempt::NotInstalled => Error::Tunnel("the host did not install".to_owned()),
            Attempt::Restarting => Error::Tunnel("the host did not restart".to_owned()),
        })
    }

    async fn connect(&self, build: &str, folder: &str) -> Result<Tunnel, Attempt> {
        for _ in 0..10 {
            match self.attempt(build, folder).await {
                Err(Attempt::Restarting) => tokio::time::sleep(Duration::from_millis(300)).await,
                attempt => return attempt,
            }
        }

        Err(Attempt::Restarting)
    }

    async fn attempt(&self, build: &str, folder: &str) -> Result<Tunnel, Attempt> {
        let script = format!(
            "export FOGDE_HOME=\"$HOME/{folder}\"; exec \"$FOGDE_HOME/.app/bin/fogde-host-{build}\" proxy --build {build}"
        );
        let mut ssh = self
            .command(&[], &script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| Attempt::Failed(spawn_error(error)))?;
        let (Some(stdout), Some(stdin)) = (ssh.stdout.take(), ssh.stdin.take()) else {
            return Err(Attempt::Failed(Error::Tunnel(
                "ssh has no pipes".to_owned(),
            )));
        };

        let url = format!("ws://fogde/?build={build}&version={VERSION}");
        match tokio_tungstenite::client_async(url, tokio::io::join(stdout, stdin)).await {
            Ok((socket, response)) => {
                let host = response
                    .headers()
                    .get(HOST_HEADER)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| {
                        Attempt::Failed(Error::Tunnel(
                            "the server did not identify itself".to_owned(),
                        ))
                    })?;

                Ok(Tunnel {
                    socket,
                    host: host.to_owned(),
                    ssh,
                })
            }
            Err(tungstenite::Error::Http(response))
                if response.status() == StatusCode::CONFLICT =>
            {
                Err(Attempt::Restarting)
            }
            Err(tungstenite::Error::Http(response))
                if response.status() == StatusCode::UPGRADE_REQUIRED =>
            {
                Err(Attempt::Failed(Error::Outdated(self.target().to_owned())))
            }
            Err(error) => Err(self.ended(ssh, &error).await),
        }
    }

    async fn ended(&self, mut ssh: Child, error: &tungstenite::Error) -> Attempt {
        let Ok(Ok(status)) = tokio::time::timeout(Duration::from_secs(5), ssh.wait()).await else {
            return Attempt::Failed(Error::Tunnel(error.to_string()));
        };
        let mut stderr = Vec::new();
        if let Some(mut pipe) = ssh.stderr.take() {
            let _ = pipe.read_to_end(&mut stderr).await;
        }

        match status.code() {
            Some(127) => Attempt::NotInstalled,
            Some(255) => Attempt::Failed(match self.failure(&stderr) {
                Failure::Error(error) => error,
                Failure::Unknown => Error::Unreachable(self.target().to_owned()),
            }),
            _ => {
                let text = String::from_utf8_lossy(&stderr);
                let line = text.lines().rev().find(|line| !line.trim().is_empty());
                Attempt::Failed(Error::Tunnel(
                    line.map_or_else(|| error.to_string(), |line| line.trim().to_owned()),
                ))
            }
        }
    }

    async fn install(&self, binary: &[u8], build: &str, folder: &str) -> Result<(), Error> {
        let script = format!(
            "d=\"$HOME/{folder}/.app/bin\"; f=\"$d/fogde-host-{build}\"; mkdir -p \"$d\" && cat > \"$f.partial\" && chmod 700 \"$f.partial\" && mv \"$f.partial\" \"$f\""
        );
        let mut ssh = self
            .command(&[], &script)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(spawn_error)?;
        let written = match ssh.stdin.take() {
            Some(mut stdin) => stdin.write_all(binary).await,
            None => Err(std::io::Error::other("ssh has no stdin")),
        };

        let output = ssh.wait_with_output().await?;
        if !output.status.success() {
            return Err(match self.failure(&output.stderr) {
                Failure::Error(error) => error,
                Failure::Unknown => Error::Unreachable(self.target().to_owned()),
            });
        }
        written?;

        Ok(())
    }
}

fn folder(home: &Path) -> Result<&str, Error> {
    let name = home.file_name().and_then(|name| name.to_str());

    name.filter(|name| {
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    })
    .ok_or_else(|| Error::Folder(home.display().to_string()))
}

fn build(binary: &[u8]) -> String {
    hex::encode(&Sha256::digest(binary)[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_builds_by_content() {
        assert_eq!(build(b"fogde"), build(b"fogde"));
        assert_ne!(build(b"fogde"), build(b"fogde2"));
        assert_eq!(build(b"fogde").len(), 16);
    }

    #[test]
    fn uses_the_folder_name_only_when_the_shell_can_take_it() {
        assert_eq!(folder(Path::new("/home/jo/app-dev")).unwrap(), "app-dev");
        assert!(folder(Path::new("/home/jo/app dev")).is_err());
        assert!(folder(Path::new("/home/jo/app';rm")).is_err());
        assert!(folder(Path::new("/home/jo/äpp")).is_err());
    }
}
