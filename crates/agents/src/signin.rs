use std::io;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use tokio::time::Instant;

use crate::agent::{command, installed, login, pastes};
use crate::{Agent, Error};

const PASTE: &str = "Paste code here if prompted >";

const PROMPT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prompt {
    Browser,
    Code { code: String, url: String },
    Paste { url: String },
}

pub struct SignIn {
    pub prompt: Prompt,
    agent: Agent,
    child: Child,
    stdin: Option<ChildStdin>,
    output: Output,
}

impl SignIn {
    pub async fn start(agent: Agent, remote: bool) -> Result<Option<Self>, Error> {
        let Some(args) = login(agent, remote) else {
            return if installed(agent).await {
                Ok(None)
            } else {
                Err(Error::Missing(agent))
            };
        };
        let paste = remote && pastes(agent);
        let mut child = command(agent)
            .await?
            .args(args)
            .stdin(if paste { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| Error::Io(agent, error))?;

        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            return Err(Error::Io(agent, io::Error::other("missing pipe")));
        };
        let mut signin = Self {
            prompt: Prompt::Browser,
            agent,
            stdin: child.stdin.take(),
            child,
            output: Output::new(agent, stdout, stderr),
        };
        if remote {
            signin.prompt = signin.read_prompt(paste).await?;
        }

        Ok(Some(signin))
    }

    pub async fn finish(mut self, code: Option<&str>) -> Result<(), Error> {
        let pasted = match (self.stdin.take(), code) {
            (Some(mut stdin), Some(code)) => {
                let line = format!("{}\n", code.trim());
                stdin
                    .write_all(line.as_bytes())
                    .await
                    .map_err(|error| Error::Io(self.agent, error))?;
                true
            }
            (None, None) => false,
            _ => {
                return Err(Error::Failed(
                    self.agent,
                    "the sign-in did not ask for a code".to_owned(),
                ));
            }
        };

        let mut last = String::new();
        while let Some(line) = self.output.next().await? {
            let line = line.trim_start_matches(PASTE).trim();
            if !line.is_empty() {
                line.clone_into(&mut last);
            }
        }
        let status = self
            .child
            .wait()
            .await
            .map_err(|error| Error::Io(self.agent, error))?;
        if status.success() {
            Ok(())
        } else if pasted {
            Err(Error::Code(self.agent, last))
        } else {
            Err(Error::Failed(self.agent, last))
        }
    }

    async fn read_prompt(&mut self, paste: bool) -> Result<Prompt, Error> {
        let deadline = Instant::now() + PROMPT;
        let mut reader = Reader::default();
        let mut transcript = String::new();
        while let Ok(line) = tokio::time::timeout_at(deadline, self.output.next()).await {
            let Some(line) = line? else {
                break;
            };
            reader.read(&line);
            transcript.push_str(&line);
            transcript.push('\n');

            let prompt = match (&reader.url, &reader.code) {
                (Some(url), _) if paste => Prompt::Paste { url: url.clone() },
                (Some(url), Some(code)) => Prompt::Code {
                    code: code.clone(),
                    url: url.clone(),
                },
                _ => continue,
            };

            return Ok(prompt);
        }

        let transcript = transcript.trim();
        Err(Error::Failed(
            self.agent,
            if transcript.is_empty() {
                "it did not show a sign-in link".to_owned()
            } else {
                transcript.to_owned()
            },
        ))
    }
}

#[derive(Default)]
struct Reader {
    url: Option<String>,
    code: Option<String>,
    code_next: bool,
}

impl Reader {
    fn read(&mut self, line: &str) {
        let line = line.trim();
        if self.code_next && !line.is_empty() {
            self.code = Some(line.to_owned());
            self.code_next = false;
        }
        if line.contains("one-time code") {
            self.code_next = true;
        }
        if self.url.is_none() {
            self.url = line
                .split_whitespace()
                .find(|word| word.starts_with("https://"))
                .map(str::to_owned);
        }
    }
}

struct Output {
    stdout: Option<Lines<BufReader<ChildStdout>>>,
    stderr: Option<Lines<BufReader<ChildStderr>>>,
    agent: Agent,
}

impl Output {
    fn new(agent: Agent, stdout: ChildStdout, stderr: ChildStderr) -> Self {
        Self {
            stdout: Some(BufReader::new(stdout).lines()),
            stderr: Some(BufReader::new(stderr).lines()),
            agent,
        }
    }

    async fn next(&mut self) -> Result<Option<String>, Error> {
        loop {
            let line = match (&mut self.stdout, &mut self.stderr) {
                (None, None) => return Ok(None),
                (Some(stdout), None) => (stdout.next_line().await, true),
                (None, Some(stderr)) => (stderr.next_line().await, false),
                (Some(stdout), Some(stderr)) => tokio::select! {
                    line = stdout.next_line() => (line, true),
                    line = stderr.next_line() => (line, false),
                },
            };
            match line {
                (Ok(Some(line)), _) => return Ok(Some(plain(&line))),
                (Ok(None), true) => self.stdout = None,
                (Ok(None), false) => self.stderr = None,
                (Err(error), _) => return Err(Error::Io(self.agent, error)),
            }
        }
    }
}

fn plain(line: &str) -> String {
    let mut plain = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(char) = chars.next() {
        if char != '\u{1b}' {
            plain.push(char);
            continue;
        }

        match chars.next() {
            Some('[') => {
                chars.by_ref().find(|char| ('@'..='~').contains(char));
            }
            Some(']') => {
                while let Some(char) = chars.next() {
                    if char == '\u{7}' {
                        break;
                    }
                    if char == '\u{1b}' {
                        chars.next();
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    plain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(lines: &[&str]) -> Reader {
        let mut reader = Reader::default();
        for line in lines {
            reader.read(&plain(line));
        }

        reader
    }

    #[test]
    fn reads_codex_device_code() {
        let reader = read(&[
            "Welcome to Codex [v\u{1b}[90m0.163.0\u{1b}[0m]",
            "Follow these steps to sign in with ChatGPT using device code authorization:",
            "",
            "1. Open this link in your browser and sign in to your account",
            "   \u{1b}[94mhttps://auth.openai.com/codex/device\u{1b}[0m",
            "",
            "2. Enter this one-time code \u{1b}[90m(expires in 15 minutes)\u{1b}[0m",
            "   \u{1b}[94m417X-5JA26\u{1b}[0m",
        ]);

        assert_eq!(
            reader.url.as_deref(),
            Some("https://auth.openai.com/codex/device")
        );
        assert_eq!(reader.code.as_deref(), Some("417X-5JA26"));
    }

    #[test]
    fn reads_claude_link() {
        let reader = read(&[
            "Opening browser to sign in…",
            "If the browser didn't open, visit: https://claude.com/cai/oauth/authorize?code=true&state=abc",
        ]);

        assert_eq!(
            reader.url.as_deref(),
            Some("https://claude.com/cai/oauth/authorize?code=true&state=abc")
        );
        assert_eq!(reader.code, None);
    }

    #[test]
    fn reads_links_wrapped_as_hyperlinks() {
        let reader = read(&[
            "Visit \u{1b}]8;;https://claude.com/cai/oauth/authorize?state=abc\u{1b}\\https://claude.com/cai/oauth/authorize?state=abc\u{1b}]8;;\u{1b}\\ to sign in",
            "\u{1b}]8;id=1;https://auth.openai.com/codex/device\u{7}https://auth.openai.com/codex/device\u{1b}]8;;\u{7}",
        ]);

        assert_eq!(
            reader.url.as_deref(),
            Some("https://claude.com/cai/oauth/authorize?state=abc")
        );
        assert_eq!(
            plain("\u{1b}]8;id=1;https://a.example\u{7}https://a.example\u{1b}]8;;\u{7}"),
            "https://a.example"
        );
    }
}
