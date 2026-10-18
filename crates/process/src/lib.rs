use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

use tokio::process::Command;

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command.kill_on_drop(true);

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

pub fn find(name: &str) -> Option<PathBuf> {
    find_in(env::split_paths(&env::var_os("PATH")?), name)
}

pub fn find_in(dirs: impl IntoIterator<Item = PathBuf>, name: &str) -> Option<PathBuf> {
    let names: Vec<String> = if cfg!(windows) {
        vec![format!("{name}.exe"), format!("{name}.cmd")]
    } else {
        vec![name.to_owned()]
    };

    dirs.into_iter()
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}
