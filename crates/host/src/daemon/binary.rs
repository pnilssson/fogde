use std::io;
use std::path::{Path, PathBuf};

use tokio::fs;

const PREFIX: &str = "fogde-host-";
const LINK: &str = "fogde-host";

pub async fn prune(bin: &Path, build: &str) {
    let Ok(mut entries) = fs::read_dir(bin).await else {
        return;
    };
    let current = format!("{PREFIX}{build}");

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(PREFIX) && !name.ends_with(".partial") && name != current {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

fn link(exe: &Path) -> Option<PathBuf> {
    let name = exe.file_name()?.to_str()?;

    name.starts_with(PREFIX).then(|| exe.with_file_name(LINK))
}

pub async fn relink(exe: &Path) -> io::Result<()> {
    let (Some(link), Some(name)) = (link(exe), exe.file_name()) else {
        return Ok(());
    };
    let partial = link.with_extension("partial");
    let _ = fs::remove_file(&partial).await;
    fs::symlink(name, &partial).await?;

    fs::rename(&partial, &link).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keeps_a_stable_link_across_updates() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("fogde-host-old");
        let new = dir.path().join("fogde-host-new");
        std::fs::write(&old, "old").unwrap();
        std::fs::write(&new, "new").unwrap();

        relink(&old).await.unwrap();
        let stable = link(&old).unwrap();
        relink(&new).await.unwrap();
        prune(dir.path(), "new").await;

        assert_eq!(std::fs::read_to_string(&stable).unwrap(), "new");
        assert!(!old.exists());
        assert_eq!(link(Path::new("/usr/bin/app")), None);
    }
}
