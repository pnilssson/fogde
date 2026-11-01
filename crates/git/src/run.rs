use std::ffi::OsStr;
use std::path::Path;
use std::process::{Output, Stdio};

use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::Error;

pub async fn git<I, S>(repo: &Path, args: I) -> Result<Vec<u8>, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    git_env(repo, args, &[]).await
}

pub async fn git_env<I, S>(repo: &Path, args: I, env: &[(&str, &OsStr)]) -> Result<Vec<u8>, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = command(repo, args)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(process::not_found(Error::Missing))?;
    if !output.status.success() {
        return Err(failure(&output));
    }

    Ok(output.stdout)
}

pub async fn git_tree<I, S>(repo: &Path, args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = command(repo, args);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = process::tree(command).map_err(process::not_found(Error::Missing))?;
    let mut stderr = Vec::new();
    let mut pipe = child.stderr();
    let read = async {
        if let Some(pipe) = pipe.as_mut() {
            let _ = pipe.read_to_end(&mut stderr).await;
        }
    };
    let (status, ()) = tokio::join!(child.wait(), read);
    let status = status?;
    if !status.success() {
        return Err(failure(&Output {
            status,
            stdout: Vec::new(),
            stderr,
        }));
    }

    Ok(())
}

pub async fn git_with_input<I, S>(repo: &Path, args: I, input: &[u8]) -> Result<Output, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    process::feed(&mut command(repo, args), input)
        .await
        .map_err(process::not_found(Error::Missing))
}

pub fn failure(output: &Output) -> Error {
    let text = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let line = lines
        .iter()
        .find_map(|line| {
            let line = line.strip_prefix("remote: ").unwrap_or(line);
            line.strip_prefix("fatal: ")
                .or_else(|| line.strip_prefix("error: "))
        })
        .or_else(|| {
            lines.iter().copied().find(|line| {
                !line.starts_with("warning: ")
                    && !line.starts_with("hint: ")
                    && !line.starts_with("To ")
            })
        })
        .or_else(|| lines.first().copied());

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
        .envs(process::UNPROMPTED);

    command
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn reports_the_error_line_rather_than_progress_or_warnings() {
        let output = |stderr: &str| Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        };
        let rejected = "To https://github.com/acme/app.git
 ! [rejected]        main -> main (fetch first)
error: failed to push some refs to 'https://github.com/acme/app.git'
hint: Updates were rejected because the remote contains work that you do not
";
        let warned = "warning: in the working copy of 'app.txt', LF will be replaced by CRLF the next time Git touches it
fatal: cannot do a partial commit during a merge.
";
        let protected = "remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: error: Changes must be made through a pull request.
To https://github.com/acme/app.git
 ! [remote rejected] main -> main (protected branch hook declined)
error: failed to push some refs to 'https://github.com/acme/app.git'
";

        assert_eq!(
            failure(&output(rejected)).to_string(),
            "failed to push some refs to 'https://github.com/acme/app.git'"
        );
        assert_eq!(
            failure(&output(warned)).to_string(),
            "cannot do a partial commit during a merge."
        );
        assert_eq!(
            failure(&output(protected)).to_string(),
            "GH006: Protected branch update failed for refs/heads/main."
        );
        assert_eq!(
            failure(&output("Already on 'main'\n")).to_string(),
            "Already on 'main'"
        );
        assert_eq!(failure(&output("")).to_string(), "Git failed");
    }

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
