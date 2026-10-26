use std::io;
use std::process::Command;

pub fn installed() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    Command::new("dpkg-query")
        .arg("-S")
        .arg(exe)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn valid(bytes: &[u8]) -> bool {
    bytes.starts_with(b"!<arch>\n") && bytes.get(8..21) == Some(b"debian-binary".as_slice())
}

pub fn install(bytes: &[u8]) -> io::Result<()> {
    if !valid(bytes) {
        return Err(io::Error::other("The update is not a Debian package."));
    }

    let folder = tempfile::Builder::new().prefix("fogde-update").tempdir()?;
    let path = folder.path().join("fogde.deb");
    std::fs::write(&path, bytes)?;
    let status = Command::new("pkexec")
        .arg("dpkg")
        .arg("-i")
        .arg(&path)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(
            "The package install was cancelled or failed.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::valid;

    #[test]
    fn accepts_only_debian_packages() {
        let mut package =
            b"!<arch>\ndebian-binary   1700000000  0     0     100644  4         `\n2.0\n".to_vec();
        assert!(valid(&package));
        package[8] = b'x';
        assert!(!valid(&package));
        assert!(!valid(b"\x7fELF"));
    }
}
