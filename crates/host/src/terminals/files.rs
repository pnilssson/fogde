use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::terminals::FILE_LIMIT;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::sync::Mutex;

const KEPT: Duration = Duration::from_hours(7 * 24);
const LONGEST: usize = 64;

static FILES: Mutex<()> = Mutex::const_new(());

pub async fn save(dir: &Path, name: &str, data: String) -> io::Result<PathBuf> {
    if data.len() > FILE_LIMIT.div_ceil(3) * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("the file is larger than {} MB", FILE_LIMIT / 1024 / 1024),
        ));
    }
    let (bytes, hash) = tokio::task::spawn_blocking(move || {
        STANDARD.decode(data).map(|bytes| {
            let hash = hash(&bytes);
            (bytes, hash)
        })
    })
    .await
    .map_err(io::Error::other)?
    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    create(dir).await?;
    let path = dir.join(format!("{hash}-{}", clean(name)));
    let stored = {
        let _files = FILES.lock().await;
        store(&path, &hash, dir, bytes).await
    };
    let old = dir.to_owned();
    tokio::spawn(async move { prune(&old).await });

    stored.map(|()| path)
}

async fn store(path: &Path, hash: &str, dir: &Path, bytes: Vec<u8>) -> io::Result<()> {
    if refresh(path).await {
        return Ok(());
    }

    let mut suffix = [0u8; 8];
    getrandom::fill(&mut suffix).map_err(io::Error::other)?;
    let temp = dir.join(format!("{hash}.{}.tmp", hex::encode(suffix)));
    fs::write(&temp, bytes).await?;
    if let Err(error) = fs::rename(&temp, path).await {
        let _ = fs::remove_file(&temp).await;
        if !fs::try_exists(path).await.unwrap_or(false) {
            return Err(error);
        }
    }

    Ok(())
}

async fn refresh(path: &Path) -> bool {
    let path = path.to_owned();

    tokio::task::spawn_blocking(move || {
        std::fs::File::options()
            .write(true)
            .open(path)?
            .set_modified(SystemTime::now())
    })
    .await
    .is_ok_and(|touched| touched.is_ok())
}

pub fn paste(path: &Path) -> String {
    let path = path.display().to_string();
    let path = if path.contains(char::is_whitespace) {
        format!("\"{path}\"")
    } else {
        path
    };

    format!("\x1b[200~{path}\x1b[201~")
}

async fn create(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);

    builder.create(dir).await
}

pub async fn prune(dir: &Path) {
    let _files = FILES.lock().await;
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return;
    };
    let Some(cutoff) = SystemTime::now().checked_sub(KEPT) else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let old = entry
            .metadata()
            .await
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| modified < cutoff);
        if old {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(&Sha256::digest(bytes)[..8])
}

fn clean(name: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let safe: Vec<char> = last
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-') {
                char
            } else {
                '-'
            }
        })
        .collect();
    let kept: String = safe[safe.len().saturating_sub(LONGEST)..].iter().collect();
    let kept = kept.trim_end_matches('.');

    if kept.is_empty() {
        "file".to_owned()
    } else {
        kept.to_owned()
    }
}

#[cfg(test)]
mod tests;
