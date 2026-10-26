use std::io;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::agent::parentless;
use crate::{Agent, Error};

mod claude;
mod codex;

pub use claude::claude;
pub use codex::codex;

const WAIT: Duration = Duration::from_secs(30);

type Answer = Option<Result<Vec<Limit>, String>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limit {
    pub minutes: i64,
    pub used: f64,
    pub resets: i64,
}

pub(super) async fn ask(
    agent: Agent,
    args: &[&str],
    requests: &str,
    reply: fn(&str) -> Answer,
) -> Result<Vec<Limit>, Error> {
    let mut child = parentless(agent, args)
        .await?
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| Error::Io(agent, error))?;
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        return Err(Error::Io(agent, io::Error::other("missing pipe")));
    };

    stdin
        .write_all(requests.as_bytes())
        .await
        .map_err(|error| Error::Io(agent, error))?;

    let mut lines = BufReader::new(stdout).lines();
    let read = async {
        while let Some(line) = lines.next_line().await? {
            if let Some(reply) = reply(&line) {
                return Ok(reply);
            }
        }

        Err(io::Error::other(format!(
            "{agent:?} closed before it sent the limits"
        )))
    };
    let reply = tokio::time::timeout(WAIT, read)
        .await
        .map_err(|_| Error::Failed(agent, format!("{agent:?} did not send the limits in time")))?
        .map_err(|error| Error::Io(agent, error))?;
    drop(stdin);

    reply.map_err(|message| Error::Failed(agent, message))
}
