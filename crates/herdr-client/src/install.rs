use std::env;
use std::path::{Path, PathBuf};

use download::Packing;

use crate::Error;

pub const VERSION: &str = "0.9.5";
const RELEASES: &str = "https://github.com/herdrdev/herdr/releases/download";

pub async fn locate(tools: &Path) -> Result<PathBuf, Error> {
    let (name, sha256, packing) = asset().ok_or(Error::Unsupported)?;
    let dir = tools.join(format!("herdr-{VERSION}"));
    let program = dir.join(format!("herdr{}", env::consts::EXE_SUFFIX));
    if program.is_file() {
        return Ok(program);
    }

    let url = format!("{RELEASES}/v{VERSION}/{name}");
    download::install(&url, sha256, packing, &dir).await?;

    Ok(program)
}

fn asset() -> Option<(&'static str, &'static str, Packing)> {
    let asset = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => (
            "herdr-windows-x86_64.zip",
            "d2c6c9c971c43a0bfdde8c692a9a1a7a45c3e4807cd01ae45662b4734fac1cbf",
            Packing::Archive,
        ),
        ("macos", "x86_64") => (
            "herdr-macos-x86_64",
            "5fb0a908ec7f6745e3e245738b06bfad56c46391c391ab3c0955fbead01f1369",
            Packing::Binary("herdr"),
        ),
        ("macos", "aarch64") => (
            "herdr-macos-aarch64",
            "fb77c2232a05a8035421382c894392f4174e4e38a18e48af3dd0290157c3a7e2",
            Packing::Binary("herdr"),
        ),
        ("linux", "x86_64") => (
            "herdr-linux-x86_64",
            "14d9e8bc8b4a9e0118d77c32266f76d461f09b40342eb23752960bd89f59929d",
            Packing::Binary("herdr"),
        ),
        ("linux", "aarch64") => (
            "herdr-linux-aarch64",
            "17d7536f2da50859523680ef279dffcac8e4a235f4e46fe8da6c2ec44141897c",
            Packing::Binary("herdr"),
        ),
        _ => return None,
    };

    Some(asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_this_platform() {
        let (name, sha256, _) = asset().unwrap();

        assert!(name.starts_with("herdr-"));
        assert_eq!(sha256.len(), 64);
    }
}
