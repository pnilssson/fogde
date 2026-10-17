use std::io;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin};

use crate::gh::{Gh, failure, spawn_error};
use crate::{Error, User};

const LOGIN: [&str; 10] = [
    "auth",
    "login",
    "--hostname",
    "github.com",
    "--git-protocol",
    "https",
    "--web",
    "--skip-ssh-key",
    "--scopes",
    "workflow",
];

pub struct SignIn {
    pub code: String,
    pub url: String,
    gh: Gh,
    child: Child,
    _stdin: ChildStdin,
    stderr: Lines<BufReader<ChildStderr>>,
}

impl SignIn {
    pub async fn start(gh: &Gh) -> Result<Self, Error> {
        let mut child = gh
            .command()
            .args(LOGIN)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(spawn_error)?;

        let (Some(stdin), Some(stderr)) = (child.stdin.take(), child.stderr.take()) else {
            return Err(Error::Io(io::Error::other("missing pipe")));
        };
        let mut stderr = BufReader::new(stderr).lines();

        let mut prompt = Prompt::default();
        let mut output = String::new();
        while let Some(line) = stderr.next_line().await? {
            prompt.read(&line);
            output.push_str(&line);
            output.push('\n');

            if let Prompt {
                code: Some(code),
                url: Some(url),
            } = prompt
            {
                return Ok(Self {
                    code,
                    url,
                    gh: gh.clone(),
                    child,
                    _stdin: stdin,
                    stderr,
                });
            }
        }

        Err(failure(output.as_bytes()))
    }

    pub async fn finish(mut self) -> Result<User, Error> {
        let mut output = String::new();
        while let Some(line) = self.stderr.next_line().await? {
            output.push_str(&line);
            output.push('\n');
        }

        if !self.child.wait().await?.success() {
            return Err(failure(output.as_bytes()));
        }

        let setup = self.gh.run(&["auth", "setup-git"]).await?;
        if !setup.status.success() {
            return Err(failure(&setup.stderr));
        }

        self.gh
            .user()
            .await?
            .ok_or_else(|| Error::Gh("signed in, but GitHub returned no user".to_owned()))
    }
}

#[derive(Default)]
struct Prompt {
    code: Option<String>,
    url: Option<String>,
}

impl Prompt {
    fn read(&mut self, line: &str) {
        if let Some((_, code)) = line.split_once("one-time code:") {
            self.code = Some(code.trim().to_owned());
        }

        if let Some(url) = line
            .split_whitespace()
            .find(|word| word.starts_with("https://"))
        {
            self.url = Some(url.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_code_and_url_from_gh() {
        let mut prompt = Prompt::default();
        for line in [
            "",
            "! First copy your one-time code: 95F3-7453",
            "Open this URL to continue in your web browser: https://github.com/login/device",
        ] {
            prompt.read(line);
        }

        assert_eq!(prompt.code.as_deref(), Some("95F3-7453"));
        assert_eq!(
            prompt.url.as_deref(),
            Some("https://github.com/login/device")
        );
    }
}
