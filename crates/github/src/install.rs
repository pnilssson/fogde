use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tokio::fs;

use crate::Error;

const VERSION: &str = "2.98.0";
const RELEASES: &str = "https://github.com/cli/cli/releases/download";

struct Asset {
    name: &'static str,
    sha256: &'static str,
    binary: &'static str,
}

pub async fn locate(tools: &Path) -> Result<PathBuf, Error> {
    if let Some(program) = on_path() {
        return Ok(program);
    }

    let asset = asset().ok_or(Error::Unsupported)?;
    let dir = tools.join(format!("gh-{VERSION}"));
    let program = asset
        .binary
        .split('/')
        .fold(dir.clone(), |path, part| path.join(part));
    if program.is_file() {
        return Ok(program);
    }

    install(&asset, &dir).await?;

    Ok(program)
}

fn on_path() -> Option<PathBuf> {
    let name = if cfg!(windows) { "gh.exe" } else { "gh" };

    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

async fn install(asset: &Asset, dir: &Path) -> Result<(), Error> {
    let staging = dir.with_extension("partial");
    if staging.exists() {
        fs::remove_dir_all(&staging).await?;
    }
    fs::create_dir_all(&staging).await?;

    let archive = staging.join(asset.name);
    let url = format!("{RELEASES}/v{VERSION}/{}", asset.name);
    run(
        curl(),
        &[
            OsStr::new("-fsSL"),
            OsStr::new("-o"),
            archive.as_os_str(),
            OsStr::new(&url),
        ],
    )
    .await?;

    let bytes = fs::read(&archive).await?;
    if sha256(&bytes) != asset.sha256 {
        fs::remove_dir_all(&staging).await?;
        return Err(Error::Checksum);
    }

    run(
        tar(),
        &[
            OsStr::new("-xf"),
            archive.as_os_str(),
            OsStr::new("-C"),
            staging.as_os_str(),
        ],
    )
    .await?;
    fs::remove_file(&archive).await?;
    fs::rename(&staging, dir).await?;

    Ok(())
}

async fn run(program: PathBuf, args: &[&OsStr]) -> Result<(), Error> {
    let output = process::command(&program)
        .args(args)
        .output()
        .await
        .map_err(|error| Error::Install(format!("{}: {error}", program.display())))?;

    if !output.status.success() {
        return Err(Error::Install(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
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
    let root = env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());

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

fn asset() -> Option<Asset> {
    let (name, sha256, binary) = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => (
            "gh_2.98.0_windows_amd64.zip",
            "c28c7b3b584967a05b74d9eaf7481bff24ddc34930bf2d6e442c148236561eb1",
            "bin/gh.exe",
        ),
        ("windows", "aarch64") => (
            "gh_2.98.0_windows_arm64.zip",
            "79e53db4e50b5c9594890a1c4d9dc941f6d19f7c3ca6d1f50982eda624607b9b",
            "bin/gh.exe",
        ),
        ("macos", "x86_64") => (
            "gh_2.98.0_macOS_amd64.zip",
            "734c7bbd0bc56a3974500ee9aea74d60f0e5b89be09e92b9d9148939a3a1e0e6",
            "gh_2.98.0_macOS_amd64/bin/gh",
        ),
        ("macos", "aarch64") => (
            "gh_2.98.0_macOS_arm64.zip",
            "8cfb027cc5310675f2b830eac8f9865c1155a45ffcf9757f699fdd5a22046ca4",
            "gh_2.98.0_macOS_arm64/bin/gh",
        ),
        ("linux", "x86_64") => (
            "gh_2.98.0_linux_amd64.tar.gz",
            "3b8ac6b30336802fc1a858d7c084e11cdf24ac1a761ca90b68022d7d729208de",
            "gh_2.98.0_linux_amd64/bin/gh",
        ),
        ("linux", "aarch64") => (
            "gh_2.98.0_linux_arm64.tar.gz",
            "cf689084f3a3618f7eae4a2420d335d74626d65f5e594b9828d125d69f800d86",
            "gh_2.98.0_linux_arm64/bin/gh",
        ),
        _ => return None,
    };

    Some(Asset {
        name,
        sha256,
        binary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_like_sha256sum() {
        assert_eq!(
            sha256(b"fogde"),
            "82a63d5a5f020b800e713d67c218f4570a694d9666275b6a3060471f306fd88d"
        );
    }

    #[test]
    fn pins_this_platform() {
        let asset = asset().unwrap();

        assert!(asset.name.contains(VERSION));
        assert_eq!(asset.sha256.len(), 64);
    }
}
