use std::env;
use std::path::{Path, PathBuf};

use download::Packing;

use crate::Error;

pub const VERSION: &str = "0.1.16";
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
            "3535f8db6258bae909a9f29ea15ada9133471a905adb90655838807ee2f49049",
        ),
        ("macos", "x86_64") => (
            "herdr-core-macos-x86_64.tar.gz",
            "6af081cdb35c8612c5752d3f64c6a378e6a0313968b6d4fbc4c03c95691b2872",
        ),
        ("macos", "aarch64") => (
            "herdr-core-macos-aarch64.tar.gz",
            "698d8f0c44f4e4b15fec174d52a39d57a014cfd71added44b53a0753b469d0fd",
        ),
        ("linux", "x86_64") => (
            "herdr-core-linux-x86_64.tar.gz",
            "58d6861176dc6fe905b58880d5bff07a9959be613a98e5d66b4583b04b6cfa45",
        ),
        ("linux", "aarch64") => (
            "herdr-core-linux-aarch64.tar.gz",
            "b6573137d70baff24fb346c957a9879af6bdc4c641188cdf9e47a7b2a677bc92",
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
