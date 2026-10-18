use std::path::PathBuf;
use std::process::Stdio;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::Error;
use crate::keys::Key;
use crate::ssh::{Failure, Server, spawn_error};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    Ready(System),
    Untrusted { fingerprint: String },
    KeyRejected { keys: Vec<Key> },
}

enum Probe {
    Ready(System),
    Unknown,
    KeyRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct System {
    pub triple: String,
}

struct HostKey {
    lines: String,
    fingerprint: String,
}

impl Server {
    pub async fn check(&self, trust: Option<&str>) -> Result<Check, Error> {
        match self.probe().await? {
            Probe::Ready(system) => return Ok(Check::Ready(system)),
            Probe::KeyRejected => return self.rejected().await,
            Probe::Unknown => {}
        }

        let key = self.host_key().await?;
        if trust != Some(key.fingerprint.as_str()) {
            return Ok(Check::Untrusted {
                fingerprint: key.fingerprint,
            });
        }

        self.remember(&key).await?;
        match self.probe().await? {
            Probe::Ready(system) => Ok(Check::Ready(system)),
            Probe::KeyRejected => self.rejected().await,
            Probe::Unknown => Err(Error::Unreachable(self.target().to_owned())),
        }
    }

    async fn probe(&self) -> Result<Probe, Error> {
        let output = self.run(&[], "uname -sm").await?;
        if output.status.success() {
            return self.system(&output.stdout).map(Probe::Ready);
        }

        match self.failure(&output.stderr) {
            Failure::Unknown => Ok(Probe::Unknown),
            Failure::Error(Error::Key(_)) => Ok(Probe::KeyRejected),
            Failure::Error(error) => Err(error),
        }
    }

    async fn rejected(&self) -> Result<Check, Error> {
        Ok(Check::KeyRejected {
            keys: self.offered().await?,
        })
    }

    fn system(&self, uname: &[u8]) -> Result<System, Error> {
        let uname = String::from_utf8_lossy(uname).trim().to_owned();
        let triple = match uname.split_whitespace().collect::<Vec<_>>()[..] {
            ["Linux", "x86_64" | "amd64"] => "x86_64-unknown-linux-musl",
            ["Linux", "aarch64" | "arm64"] => "aarch64-unknown-linux-musl",
            _ => return Err(Error::Unsupported(self.target().to_owned(), uname)),
        };

        Ok(System {
            triple: triple.to_owned(),
        })
    }

    async fn host_key(&self) -> Result<HostKey, Error> {
        let file = std::env::temp_dir().join(format!(
            "fogde-known-hosts-{}-{}",
            std::process::id(),
            self.target().replace(['@', ':', '/', '\\'], "-")
        ));
        let known = format!(
            "UserKnownHostsFile=\"{}\"",
            file.display().to_string().replace('\\', "/")
        );
        let options = [
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            known.as_str(),
            "-o",
            "GlobalKnownHostsFile=none",
        ];

        let output = self.run(&options, "exit").await;
        let lines = fs::read_to_string(&file).await.unwrap_or_default();
        let listed = process::command("ssh-keygen")
            .args(["-l", "-f"])
            .arg(&file)
            .stdin(Stdio::null())
            .output()
            .await;
        let _ = fs::remove_file(&file).await;

        if lines.trim().is_empty() {
            return Err(self.error(&output?.stderr));
        }
        let listed = listed.map_err(spawn_error)?;
        let fingerprint = fingerprint(&String::from_utf8_lossy(&listed.stdout))
            .ok_or_else(|| Error::Ssh("could not read the server's key".to_owned()))?;

        Ok(HostKey { lines, fingerprint })
    }

    async fn remember(&self, key: &HostKey) -> Result<(), Error> {
        let file = self.known_hosts().await?;
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir).await?;
        }

        let mut known = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .await?;
        known
            .write_all(format!("\n{}\n", key.lines.trim()).as_bytes())
            .await?;

        Ok(())
    }

    async fn known_hosts(&self) -> Result<PathBuf, Error> {
        let home = std::env::home_dir().ok_or_else(|| Error::Ssh("no home folder".to_owned()))?;
        let config = process::command("ssh")
            .args(["-G", "--", self.target()])
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(spawn_error)?;
        let config = String::from_utf8_lossy(&config.stdout);
        let files = config
            .lines()
            .find_map(|line| line.strip_prefix("userknownhostsfile "))
            .ok_or_else(|| Error::Ssh("ssh did not report its known hosts file".to_owned()))?;

        Ok(first_file(files, &home))
    }

    fn error(&self, stderr: &[u8]) -> Error {
        match self.failure(stderr) {
            Failure::Error(error) => error,
            Failure::Unknown => Error::Unreachable(self.target().to_owned()),
        }
    }
}

fn first_file(files: &str, home: &std::path::Path) -> PathBuf {
    let default = format!("{}/.ssh/known_hosts", home.display());
    if files == default || files.starts_with(&format!("{default} ")) {
        return home.join(".ssh").join("known_hosts");
    }

    let first = files.split(' ').next().unwrap_or(files);
    match first.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(first),
    }
}

fn fingerprint(listed: &str) -> Option<String> {
    listed.lines().find_map(|line| {
        line.split_whitespace()
            .nth(1)
            .filter(|fingerprint| fingerprint.starts_with("SHA256:"))
            .map(str::to_owned)
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn reads_systems() {
        let server = Server::new("my-vps").unwrap();

        assert_eq!(
            server.system(b"Linux x86_64\n").unwrap().triple,
            "x86_64-unknown-linux-musl"
        );
        assert_eq!(
            server.system(b"Linux aarch64\n").unwrap().triple,
            "aarch64-unknown-linux-musl"
        );
        assert_eq!(
            server.system(b"Darwin arm64\n").unwrap_err().to_string(),
            "my-vps runs Darwin arm64, and only 64-bit Linux servers are supported"
        );
    }

    #[test]
    fn reads_fingerprints() {
        let listed =
            "256 SHA256:Kk2CP67tlSOJX5efGAD2WESdUNd9BWHzt6QAc28oi8M 203.0.113.10 (ED25519)\n";

        assert_eq!(
            fingerprint(listed).as_deref(),
            Some("SHA256:Kk2CP67tlSOJX5efGAD2WESdUNd9BWHzt6QAc28oi8M")
        );
        assert_eq!(fingerprint(""), None);
    }

    #[test]
    fn finds_the_known_hosts_file() {
        let home = Path::new("C:/Users/Jo Doe");

        assert_eq!(
            first_file(
                "C:/Users/Jo Doe/.ssh/known_hosts C:/Users/Jo Doe/.ssh/known_hosts2",
                home
            ),
            home.join(".ssh").join("known_hosts")
        );
        assert_eq!(
            first_file("~/.ssh/fogde_hosts ~/.ssh/known_hosts2", home),
            home.join(".ssh/fogde_hosts")
        );
        assert_eq!(
            first_file("/etc/ssh/hosts", home),
            PathBuf::from("/etc/ssh/hosts")
        );
    }
}
