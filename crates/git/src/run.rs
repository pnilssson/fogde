use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

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

pub async fn git_paths(repo: &Path, names: &[&str]) -> Result<Vec<PathBuf>, Error> {
    let mut args = vec!["rev-parse"];
    for name in names {
        args.extend(["--git-path", name]);
    }
    let output = git(repo, args).await?;

    Ok(String::from_utf8_lossy(&output)
        .lines()
        .map(|line| repo.join(line))
        .collect())
}

pub async fn git_with_input<I, S>(repo: &Path, args: I, input: &[u8]) -> Result<Output, Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    process::feed(&mut command(repo, args), input)
        .await
        .map_err(spawn_error)
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
