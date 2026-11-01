use std::fs::Permissions;
use std::io;
use std::path::Path;
use std::time::Duration;

use tokio::fs;
use tokio::io::AsyncWriteExt;

const SWAP_TRIES: u32 = 5;
const SWAP_WAIT: Duration = Duration::from_millis(50);

pub(super) async fn replace(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let permissions = fs::metadata(file).await?.permissions();
    if permissions.readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the note is read-only",
        ));
    }

    let name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut suffix = [0u8; 8];
    getrandom::fill(&mut suffix).map_err(io::Error::other)?;
    let temp = file.with_file_name(format!(".{name}.{}.tmp", hex::encode(suffix)));

    let replaced = match stage(&temp, &permissions, bytes).await {
        Ok(()) => swap(&temp, file).await,
        Err(error) => Err(error),
    };
    if replaced.is_err() {
        let _ = fs::remove_file(&temp).await;
    }

    replaced
}

async fn stage(temp: &Path, permissions: &Permissions, bytes: &[u8]) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        options.mode(permissions.mode());
    }

    let mut staged = options.open(temp).await?;
    staged.write_all(bytes).await?;
    staged.sync_all().await?;

    fs::set_permissions(temp, permissions.clone()).await
}

async fn swap(temp: &Path, file: &Path) -> io::Result<()> {
    let mut tries = 0;
    loop {
        match fs::rename(temp, file).await {
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied && tries < SWAP_TRIES => {
                tries += 1;
                tokio::time::sleep(SWAP_WAIT).await;
            }
            result => {
                return result;
            }
        }
    }
}
