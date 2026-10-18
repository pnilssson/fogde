use std::io;
use std::process::{Output, Stdio};

use tokio::process::Command;

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    target: String,
}

impl Server {
    pub fn new(target: &str) -> Result<Self, Error> {
        let target = target.trim();
        if target.is_empty()
            || target.starts_with('-')
            || target.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(Error::Target(target.to_owned()));
        }

        Ok(Self {
            target: target.to_owned(),
        })
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub(crate) fn command(&self, options: &[&str], script: &str) -> Command {
        let mut command = process::command("ssh");
        command.args([
            "-T",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=3",
        ]);
        command.args(options);
        command
            .arg("--")
            .arg(&self.target)
            .arg(format!("sh -c '{script}'"));
        command.stdin(Stdio::null());

        command
    }

    pub(crate) async fn run(&self, options: &[&str], script: &str) -> Result<Output, Error> {
        self.command(options, script)
            .output()
            .await
            .map_err(spawn_error)
    }

    pub(crate) fn failure(&self, stderr: &[u8]) -> Failure {
        let text = String::from_utf8_lossy(stderr);
        let target = self.target.clone();

        if text.contains("REMOTE HOST IDENTIFICATION HAS CHANGED") {
            Failure::Error(Error::Changed(target))
        } else if text.contains("Host key verification failed") {
            Failure::Unknown
        } else if text.contains("Permission denied") {
            Failure::Error(Error::Key(target))
        } else if [
            "Could not resolve hostname",
            "Connection timed out",
            "Operation timed out",
            "Connection refused",
            "No route to host",
            "Network is unreachable",
        ]
        .iter()
        .any(|reason| text.contains(reason))
        {
            Failure::Error(Error::Unreachable(target))
        } else {
            let line = text.lines().rev().find(|line| !line.trim().is_empty());
            Failure::Error(Error::Ssh(line.unwrap_or("it stopped").trim().to_owned()))
        }
    }
}

pub(crate) enum Failure {
    Unknown,
    Error(Error),
}

pub(crate) fn spawn_error(error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::NotFound {
        return Error::Missing;
    }

    Error::Io(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reason(stderr: &str) -> String {
        match Server::new("my-vps").unwrap().failure(stderr.as_bytes()) {
            Failure::Unknown => "unknown".to_owned(),
            Failure::Error(error) => error.to_string(),
        }
    }

    #[test]
    fn accepts_hosts_and_addresses_only() {
        assert!(Server::new("my-vps").is_ok());
        assert!(Server::new(" deploy@1.2.3.4 ").is_ok());
        assert!(Server::new("-oProxyCommand=evil").is_err());
        assert!(Server::new("a b").is_err());
        assert!(Server::new("").is_err());
    }

    #[test]
    fn explains_ssh_failures() {
        assert_eq!(
            reason("deploy@1.2.3.4: Permission denied (publickey).\r\n"),
            "my-vps did not accept your SSH key"
        );
        assert_eq!(
            reason("ssh: connect to host 1.2.3.4 port 22: Connection timed out\r\n"),
            "could not reach my-vps"
        );
        assert_eq!(
            reason(
                "No ED25519 host key is known for 1.2.3.4 and you have requested strict checking.\r\nHost key verification failed.\r\n"
            ),
            "unknown"
        );
        assert_eq!(
            reason(
                "@@@\r\n@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @\r\nHost key verification failed.\r\n"
            ),
            "the identity of my-vps changed since you last connected, so the connection was stopped"
        );
    }
}
