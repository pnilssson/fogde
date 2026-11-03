use std::env;
use std::path::{Path, PathBuf};

use download::Packing;

use crate::Error;

pub const VERSION: &str = "0.1.17";
const RELEASES: &str = "https://releases.herdr.dev/core";

pub async fn locate(tools: &Path) -> Result<PathBuf, Error> {
    let (name, sha256) = asset().ok_or(Error::Unsupported)?;
    let dir = tools.join(format!("herdr-core-{VERSION}"));
    let program = dir.join(format!("herdr-core{}", env::consts::EXE_SUFFIX));
    if program.is_file() {
        return Ok(program);
    }

    let url = format!("{RELEASES}/{VERSION}/{name}");
    download::install(&url, sha256, Packing::Archive, &dir).await?;

    Ok(program)
}

fn asset() -> Option<(&'static str, &'static str)> {
    let asset = match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => (
            "herdr-core-windows-x86_64.zip",
            "a212b69db0efb808882683f15e5da45ab9606da7bcc54b97b29ed4fa540f5491",
        ),
        ("macos", "x86_64") => (
            "herdr-core-macos-x86_64.tar.gz",
            "e2dc3211edb1f1c77d0e73dd76c1426636d71050f34a5303916a56d363d7062b",
        ),
        ("macos", "aarch64") => (
            "herdr-core-macos-aarch64.tar.gz",
            "cedbd4d4d9ba1385588198dd4d3c67d2654478f33add5b6445d4d0eb9da5fcf3",
        ),
        ("linux", "x86_64") => (
            "herdr-core-linux-x86_64.tar.gz",
            "82ec44043f6894bfdb716b979b87cda43fd7fa46b0104e7b7bd93e64d1a9b877",
        ),
        ("linux", "aarch64") => (
            "herdr-core-linux-aarch64.tar.gz",
            "3a37a454ec63494cf4306db26481fe9db96673d888723701d24f8a992ae8ab8c",
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
        let (name, sha256) = asset().unwrap();

        assert!(name.starts_with("herdr-core-"));
        assert_eq!(sha256.len(), 64);
    }
}
