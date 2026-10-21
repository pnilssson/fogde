use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tokio::fs;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Failed(String),

    #[error("the download did not match its checksum")]
    Checksum,

    #[error(transparent)]
    Io(#[from] io::Error),
}

pub enum Packing {
    Archive,
    Binary(&'static str),
}

pub async fn install(url: &str, sha256: &str, packing: Packing, dir: &Path) -> Result<(), Error> {
    let mut name = dir.file_name().map(OsStr::to_owned).unwrap_or_default();
    name.push(".partial");
    let staging = dir.with_file_name(name);
    if staging.exists() {
        fs::remove_dir_all(&staging).await?;
    }
    fs::create_dir_all(&staging).await?;

    let file = match packing {
        Packing::Archive => url.rsplit('/').next().unwrap_or(url),
        Packing::Binary(name) => name,
    };
    let path = staging.join(file);
    run(
        curl(),
        &["-fsSL".into(), "-o".into(), path.clone().into(), url.into()],
    )
    .await?;

    let bytes = fs::read(&path).await?;
    if hex(&Sha256::digest(&bytes)) != sha256 {
        fs::remove_dir_all(&staging).await?;
        return Err(Error::Checksum);
    }

    match packing {
        Packing::Archive => {
            run(
                tar(),
                &[
                    "-xf".into(),
                    path.clone().into(),
                    "-C".into(),
                    staging.clone().into(),
                ],
            )
            .await?;
            fs::remove_file(&path).await?;
        }
        Packing::Binary(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).await?;
            }
        }
    }
    fs::rename(&staging, dir).await?;

    Ok(())
}

async fn run(program: PathBuf, args: &[OsString]) -> Result<(), Error> {
    let output = process::command(&program)
        .args(args)
        .output()
        .await
        .map_err(|error| Error::Failed(format!("{}: {error}", program.display())))?;

    if !output.status.success() {
        return Err(Error::Failed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

#[cfg(windows)]
fn curl() -> PathBuf {
    system32("curl.exe")
}

#[cfg(windows)]
fn tar() -> PathBuf {
    system32("tar.exe")
}

#[cfg(windows)]
fn system32(name: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());

    PathBuf::from(root).join("System32").join(name)
}

#[cfg(not(windows))]
fn curl() -> PathBuf {
    PathBuf::from("curl")
}

#[cfg(not(windows))]
fn tar() -> PathBuf {
    PathBuf::from("tar")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_like_sha256sum() {
        assert_eq!(
            hex(&Sha256::digest(b"data")),
            "3a6eb0790f39ac87c94f3856b2dd2c5d110e6811602261a9a923d3bb23adc8b7"
        );
    }
}
