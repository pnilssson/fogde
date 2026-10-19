use std::ffi::OsString;
use std::path::Path;

use tokio::fs;

use crate::Error;
use crate::run::{failure, git, git_with_input};

const DEFAULT: [&str; 2] = [".env", ".claude/settings.local.json"];

pub async fn copy(repo: &Path, worktree: &Path) -> Result<(), Error> {
    let mut args: Vec<OsString> = ["ls-files", "-z", "--others", "--ignored"]
        .into_iter()
        .map(OsString::from)
        .collect();
    let file = repo.join(".worktreeinclude");
    if fs::try_exists(&file).await? {
        let mut pattern = OsString::from("--exclude-from=");
        pattern.push(&file);
        args.push(pattern);
    } else {
        args.extend(
            DEFAULT
                .iter()
                .map(|path| OsString::from(format!("--exclude={path}"))),
        );
    }

    let listed = git(repo, args).await?;
    if listed.is_empty() {
        return Ok(());
    }

    let checked = git_with_input(repo, ["check-ignore", "-z", "--stdin"], &listed).await?;
    if !checked.status.success() && checked.status.code() != Some(1) {
        return Err(failure(&checked));
    }

    for file in checked
        .stdout
        .split(|byte| *byte == 0)
        .filter(|file| !file.is_empty() && !file.ends_with(b"/"))
    {
        let file = String::from_utf8_lossy(file);
        let target = worktree.join(file.as_ref());
        if let Some(dir) = target.parent() {
            fs::create_dir_all(dir).await?;
        }
        fs::copy(repo.join(file.as_ref()), target).await?;
    }

    Ok(())
}
