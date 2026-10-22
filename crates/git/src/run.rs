use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Output, Stdio};

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::Error;

pub async fn git<I, S>(repo: &Path, args: I) -> Result<Vec<u8>, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = command(repo, args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(spawn_error)?;
    if !output.status.success() {
        return Err(failure(&output));
    }

    Ok(output.stdout)
}

pub async fn git_with_input<I, S>(repo: &Path, args: I, input: &[u8]) -> Result<Output, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = command(repo, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_error)?;
    let stdin = child.stdin.take();
    let write = async move {
        if let Some(mut stdin) = stdin {
            stdin.write_all(input).await?;
        }

        io::Result::Ok(())
    };

    let (written, output) = tokio::join!(write, child.wait_with_output());
    let output = output?;
    if output.status.success() {
        written?;
    }

    Ok(output)
}

pub fn failure(output: &Output) -> Error {
    let text = String::from_utf8_lossy(&output.stderr);
    let line = text
        .lines()
        .map(|line| {
            line.trim()
                .trim_start_matches("fatal: ")
                .trim_start_matches("error: ")
        })
        .find(|line| !line.is_empty());

    Error::Git(line.unwrap_or("Git failed").to_owned())
}

fn command<I, S>(repo: &Path, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = process::command("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_ASKPASS", "")
        .env("SSH_ASKPASS_REQUIRE", "never");

    command
}

fn spawn_error(error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::NotFound {
        return Error::Missing;
    }

    Error::Io(error)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn reads_output_while_writing_input() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), ["init", "--quiet"]).await.unwrap();
        tokio::fs::write(dir.path().join(".gitignore"), "build/\n")
            .await
            .unwrap();
        let input: Vec<u8> = (0..40_000)
            .flat_map(|index| format!("build/{index:0>64}\0").into_bytes())
            .collect();

        let output = tokio::time::timeout(
            Duration::from_secs(30),
            git_with_input(dir.path(), ["check-ignore", "-z", "--stdin"], &input),
        )
        .await
        .expect("git stopped reading its input")
        .unwrap();

        assert_eq!(output.stdout, input);
    }
}
