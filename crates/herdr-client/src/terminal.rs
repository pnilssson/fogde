use std::io;
use std::path::Path;
use std::process::Stdio;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

use crate::Error;

pub struct Terminal {
    _child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    stderr: ChildStderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub seq: u64,
    pub width: u16,
    pub height: u16,
    pub full: bool,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scroll {
    Up,
    Down,
}

impl Terminal {
    pub fn open(
        herdr: &Path,
        socket: &Path,
        target: &str,
        cols: u16,
        rows: u16,
    ) -> Result<Self, Error> {
        let mut child = command(herdr, socket)
            .args(["terminal", "session", "control", target])
            .args(["--cols", &cols.to_string(), "--rows", &rows.to_string()])
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

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
            stderr,
        })
    }

    pub async fn next(&mut self) -> Result<Frame, Error> {
        while let Some(line) = self.stdout.next_line().await? {
            match serde_json::from_str(&line)? {
                Record::Frame(frame) => {
                    return Ok(Frame {
                        seq: frame.seq,
                        width: frame.width,
                        height: frame.height,
                        full: frame.full,
                        bytes: STANDARD.decode(frame.bytes)?,
                    });
                }
                Record::Closed {
                    reason: Some(reason),
                } => {
                    return Err(Error::Stream(reason));
                }
                Record::Closed { reason: None } => {
                    return Err(Error::Closed);
                }
                Record::Other => {}
            }
        }

        let mut reason = String::new();
        self.stderr.read_to_string(&mut reason).await?;
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(Error::Closed);
        }

        Err(Error::Stream(reason.to_owned()))
    }

    pub async fn input(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.send(&Control::Input {
            bytes: STANDARD.encode(bytes),
        })
        .await
    }

    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Error> {
        self.send(&Control::Resize { cols, rows }).await
    }

    pub async fn scroll(
        &mut self,
        direction: Scroll,
        lines: u16,
        column: u16,
        row: u16,
    ) -> Result<(), Error> {
        self.send(&Control::Scroll {
            direction,
            lines,
            column,
            row,
        })
        .await
    }

    async fn send(&mut self, control: &Control) -> Result<(), Error> {
        let mut line = serde_json::to_vec(control)?;
        line.push(b'\n');
        self.stdin.write_all(&line).await?;

        Ok(())
    }
}

fn command(herdr: &Path, socket: &Path) -> Command {
    let mut command = process::command(herdr);
    command
        .env("HERDR_SOCKET_PATH", socket)
        .env_remove("HERDR_SESSION");

    command
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Record {
    #[serde(rename = "terminal.frame")]
    Frame(WireFrame),
    #[serde(rename = "terminal.closed")]
    Closed { reason: Option<String> },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct WireFrame {
    seq: u64,
    width: u16,
    height: u16,
    full: bool,
    bytes: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum Control {
    #[serde(rename = "terminal.input")]
    Input { bytes: String },
    #[serde(rename = "terminal.resize")]
    Resize { cols: u16, rows: u16 },
    #[serde(rename = "terminal.scroll")]
    Scroll {
        direction: Scroll,
        lines: u16,
        column: u16,
        row: u16,
    },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn reads_frame() {
        let record: Record = serde_json::from_str(
            r#"{"type":"terminal.frame","seq":2,"encoding":"ansi","width":80,"height":24,"full":false,"bytes":"aGk="}"#,
        )
        .unwrap();

        let Record::Frame(frame) = record else {
            panic!("expected a frame");
        };
        assert_eq!(
            (frame.seq, frame.width, frame.height, frame.full),
            (2, 80, 24, false)
        );
        assert_eq!(STANDARD.decode(frame.bytes).unwrap(), b"hi");
    }

    #[test]
    fn reads_closed_and_ignores_new_records() {
        let closed: Record = serde_json::from_str(
            r#"{"type":"terminal.closed","reason":"terminal target w9:p9 not found"}"#,
        )
        .unwrap();
        let other: Record = serde_json::from_str(r#"{"type":"terminal.bell"}"#).unwrap();

        assert!(matches!(
            closed,
            Record::Closed { reason: Some(reason) } if reason.ends_with("not found")
        ));
        assert!(matches!(other, Record::Other));
    }

    #[test]
    fn writes_controls() {
        let controls = [
            Control::Input {
                bytes: STANDARD.encode("hi"),
            },
            Control::Resize {
                cols: 120,
                rows: 40,
            },
            Control::Scroll {
                direction: Scroll::Up,
                lines: 3,
                column: 4,
                row: 5,
            },
        ]
        .map(|control| serde_json::to_value(control).unwrap());

        assert_eq!(
            controls,
            [
                json!({"type": "terminal.input", "bytes": "aGk="}),
                json!({"type": "terminal.resize", "cols": 120, "rows": 40}),
                json!({"type": "terminal.scroll", "direction": "up", "lines": 3, "column": 4, "row": 5}),
            ]
        );
    }
}
