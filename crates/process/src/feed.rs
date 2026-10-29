use std::io;
use std::process::{Output, Stdio};

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn feed(command: &mut Command, input: &[u8]) -> io::Result<Output> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
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
