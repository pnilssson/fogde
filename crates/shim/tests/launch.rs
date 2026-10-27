use std::env::consts::EXE_SUFFIX;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn launcher(dir: &Path) -> PathBuf {
    dir.join(format!("codex{EXE_SUFFIX}"))
}

fn install(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::copy(env!("CARGO_BIN_EXE_codex"), launcher(dir))?;

    Ok(())
}

#[cfg(windows)]
fn fake(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;

    fs::write(dir.join("codex.cmd"), "@echo %*\r\n@exit /b 3\r\n")
}

#[cfg(unix)]
fn fake(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(dir)?;
    let path = dir.join("codex");
    fs::write(&path, "#!/bin/sh\necho \"$@\"\nexit 3\n")?;

    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
}

fn run(root: &Path, dirs: &[&str], args: &[&str]) -> io::Result<Output> {
    let path =
        std::env::join_paths(dirs.iter().map(|dir| root.join(dir))).map_err(io::Error::other)?;

    Command::new(launcher(&root.join(dirs[0])))
        .args(args)
        .env("PATH", path)
        .output()
}

fn printed(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn runs_the_next_codex_on_the_path_without_the_shared_server() {
    let root = tempfile::tempdir().unwrap();
    install(&root.path().join("shims")).unwrap();
    fake(&root.path().join("bin")).unwrap();

    let output = run(root.path(), &["shims", "bin"], &["fix", "login"]).unwrap();

    assert_eq!(printed(&output), "--no-daemon fix login");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn leaves_commands_that_need_the_shared_server_alone() {
    let root = tempfile::tempdir().unwrap();
    install(&root.path().join("shims")).unwrap();
    fake(&root.path().join("bin")).unwrap();

    for args in [
        &["agents"][..],
        &["--no-daemon", "fix"],
        &["queue", "--help"],
    ] {
        let output = run(root.path(), &["shims", "bin"], args).unwrap();

        assert_eq!(printed(&output), args.join(" "));
    }
}

#[test]
fn passes_through_another_launcher_earlier_on_the_path() {
    let root = tempfile::tempdir().unwrap();
    install(&root.path().join("shims")).unwrap();
    install(&root.path().join("other")).unwrap();
    fake(&root.path().join("bin")).unwrap();

    let output = run(root.path(), &["other", "shims", "bin"], &["fix"]).unwrap();

    assert_eq!(printed(&output), "--no-daemon fix");
}

#[test]
fn says_so_when_codex_is_not_installed() {
    let root = tempfile::tempdir().unwrap();
    install(&root.path().join("shims")).unwrap();

    let output = run(root.path(), &["shims"], &["fix"]).unwrap();

    assert_eq!(output.status.code(), Some(127));
    assert!(String::from_utf8_lossy(&output.stderr).contains("codex was not found"));
}
