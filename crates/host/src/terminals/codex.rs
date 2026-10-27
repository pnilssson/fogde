use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::fs;

use super::launch::path_with;

const FUNCTION: &str = include_str!("codex/fogde-codex.sh");

#[derive(Default)]
pub struct Wrap {
    vars: BTreeMap<String, String>,
    bin: Option<PathBuf>,
}

impl Wrap {
    pub fn pane_env(
        &self,
        base: &BTreeMap<String, String>,
        gh: Option<&Path>,
    ) -> BTreeMap<String, String> {
        let mut env = base.clone();
        env.extend(self.vars.clone());
        let first: Vec<PathBuf> = gh
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .into_iter()
            .chain(self.bin.clone())
            .collect();
        if !first.is_empty() {
            env.insert("PATH".to_owned(), path_with(&first));
        }

        env
    }
}

pub async fn prepare(dir: &Path, launcher: Option<&Path>) -> Wrap {
    let mut wrap = Wrap::default();
    if let Some(function) = FUNCTION.strip_prefix("codex") {
        wrap.vars
            .insert("BASH_FUNC_codex%%".to_owned(), function.to_owned());
    }

    if let Some(launcher) = launcher {
        let bin = dir.join("bin");
        match install(launcher, &bin).await {
            Ok(()) => wrap.bin = Some(bin),
            Err(error) => tracing::warn!(%error, "could not install the codex launcher"),
        }
    }

    #[cfg(unix)]
    if let Err(error) = unix::prepare(dir, &mut wrap.vars, unix::var).await {
        tracing::warn!(%error, "could not set up codex for zsh and fish");
    }

    wrap
}

async fn install(launcher: &Path, bin: &Path) -> io::Result<()> {
    let name = launcher
        .file_name()
        .ok_or_else(|| io::Error::other("the launcher has no file name"))?;
    prune(bin).await;

    put(&bin.join(name), &fs::read(launcher).await?).await
}

async fn prune(bin: &Path) {
    let Ok(mut entries) = fs::read_dir(bin).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "old")
        {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

async fn put(path: &Path, content: &[u8]) -> io::Result<()> {
    if fs::read(path).await.is_ok_and(|current| current == content) {
        return Ok(());
    }

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).await?;
    }
    let partial = path.with_extension("partial");
    fs::write(&partial, content).await?;
    if fs::rename(&partial, path).await.is_ok() {
        return Ok(());
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    fs::rename(path, path.with_extension(format!("{stamp}.old"))).await?;

    fs::rename(&partial, path).await
}

#[cfg(unix)]
mod unix {
    use std::collections::BTreeMap;
    use std::io;
    use std::path::Path;

    use super::{FUNCTION, put};

    const ZSHENV: &str = include_str!("codex/zshenv.zsh");
    const FISH: &str = include_str!("codex/fogde.fish");
    const DATA: &str = "/usr/local/share:/usr/share";

    pub(super) async fn prepare(
        dir: &Path,
        env: &mut BTreeMap<String, String>,
        var: impl Fn(&str) -> Option<String>,
    ) -> io::Result<()> {
        let zsh = dir.join("zsh");
        put(&zsh.join(".zshenv"), ZSHENV.as_bytes()).await?;
        put(&zsh.join("fogde-codex.sh"), FUNCTION.as_bytes()).await?;
        put(
            &dir.join("fish").join("vendor_conf.d").join("fogde.fish"),
            FISH.as_bytes(),
        )
        .await?;

        env.insert("ZDOTDIR".to_owned(), zsh.to_string_lossy().into_owned());
        if let Some(original) =
            var("ZDOTDIR").filter(|original| !Path::new(original).join("fogde-codex.sh").exists())
        {
            env.insert("FOGDE_ZDOTDIR".to_owned(), original);
        }

        let data = var("XDG_DATA_DIRS");
        env.insert(
            "XDG_DATA_DIRS".to_owned(),
            format!("{}:{}", dir.display(), data.as_deref().unwrap_or(DATA)),
        );
        if let Some(data) = data {
            env.insert("FOGDE_XDG_DATA_DIRS".to_owned(), data);
        }

        Ok(())
    }

    pub(super) fn var(name: &str) -> Option<String> {
        std::env::var(name).ok().filter(|value| !value.is_empty())
    }
}

#[cfg(test)]
mod tests;
