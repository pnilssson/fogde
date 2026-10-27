use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const SHARED: [&str; 5] = [
    "agents",
    "queue",
    "remote-control",
    "--remote",
    "--no-daemon",
];

fn main() {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let Some(codex) = codex() else {
        fail(127, "codex was not found on PATH");
    };

    #[cfg(windows)]
    if let Err(error) = ctrlc::set_handler(|| {}) {
        fail(1, &format!("could not keep Ctrl+C for codex: {error}"));
    }

    match Command::new(codex).args(flags(&args)).args(&args).status() {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => fail(126, &format!("could not start codex: {error}")),
    }
}

fn flags(args: &[OsString]) -> &'static [&'static str] {
    let shared = args
        .iter()
        .filter_map(|arg| arg.to_str())
        .any(|arg| SHARED.contains(&arg) || arg.starts_with("--remote="));
    if shared {
        return &[];
    }

    &["--no-daemon"]
}

fn codex() -> Option<PathBuf> {
    let own = env::current_exe().ok()?;
    let dirs: Vec<PathBuf> = env::split_paths(&env::var_os("PATH")?).collect();
    let after = own
        .parent()
        .and_then(|parent| dirs.iter().position(|dir| same(dir, parent)))
        .map_or(0, |index| index + 1);
    let names = names();

    dirs[after..]
        .iter()
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file() && !same(candidate, &own))
}

fn names() -> Vec<String> {
    if !cfg!(windows) {
        return vec!["codex".to_owned()];
    }

    env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned())
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| format!("codex{extension}"))
        .collect()
}

fn same(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn fail(code: i32, message: &str) -> ! {
    let _ = writeln!(io::stderr(), "codex: {message}");
    process::exit(code);
}
