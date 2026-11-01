use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError};

use github::Gh;
use herdr_client::Herdr;
use remote::{Binaries, Tunnel};
use tokio::fs;
use tokio::sync::{Mutex, OnceCell};

use crate::db::Db;
use crate::monitor::Monitor;
use crate::notes::Notes;
use crate::settings::Settings;
use crate::terminals::codex::{self, Wrap};
use crate::terminals::{self, Tracker};
use crate::usage::Meter;
use crate::workspaces::{Searches, Watch};

pub struct Shared {
    pub settings: Settings,
    pub db: Db,
    pub binaries: Option<Binaries>,
    pub tunnels: Mutex<HashMap<String, Tunnel>>,
    pub placing: Mutex<()>,
    pub searches: Searches,
    pub tracker: Arc<Tracker>,
    pub watch: Watch,
    pub notes: Notes,
    pub usage: Meter,
    pub monitor: Monitor,
    fogde: PathBuf,
    launcher: Option<PathBuf>,
    codex: Mutex<()>,
    tools: PathBuf,
    gh: OnceCell<Gh>,
    herdr: OnceCell<Arc<Herdr>>,
    hosts: std::sync::Mutex<Vec<String>>,
}

impl Shared {
    pub fn new(binaries: Option<Binaries>, launcher: Option<PathBuf>) -> io::Result<Self> {
        let fogde = fogde()?;
        let app = app(&fogde)?;

        let shared = Self {
            settings: Settings::new(&app),
            db: Db::new(app.join("fogde.db")),
            binaries,
            tunnels: Mutex::default(),
            placing: Mutex::default(),
            searches: Searches::default(),
            tracker: Arc::default(),
            watch: Watch::default(),
            notes: Notes::default(),
            usage: Meter::default(),
            monitor: Monitor::default(),
            tools: app.join("tools"),
            fogde,
            launcher,
            codex: Mutex::default(),
            gh: OnceCell::new(),
            herdr: OnceCell::new(),
            hosts: std::sync::Mutex::default(),
        };
        let images = shared.images();
        tokio::spawn(async move { terminals::prune(&images).await });

        Ok(shared)
    }

    pub async fn gh(&self) -> Result<&Gh, github::Error> {
        self.gh.get_or_try_init(|| Gh::locate(&self.tools)).await
    }

    pub async fn accounts(&self) -> Result<Vec<github::Account>, github::Error> {
        let accounts = self.gh().await?.accounts().await?;
        *self.hosts.lock().unwrap_or_else(PoisonError::into_inner) = accounts
            .iter()
            .map(|account| account.host.clone())
            .collect();

        Ok(accounts)
    }

    pub fn hosts(&self) -> Vec<String> {
        self.hosts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub async fn herdr(&self) -> Result<&Arc<Herdr>, herdr_client::Error> {
        let home = self.fogde.join(".app").join("herdr-core");

        self.herdr
            .get_or_try_init(|| async move {
                let herdr = Arc::new(Herdr::start(&self.tools, &home).await?);
                tokio::spawn(Arc::clone(&self.tracker).run(Arc::clone(&herdr), self.turns()));
                if let Ok(stats) = self.settings.stats().await {
                    self.tracker.set_stats(stats);
                }
                tokio::spawn(terminals::count(Arc::clone(&self.tracker)));

                Ok(herdr)
            })
            .await
    }

    pub async fn codex(&self) -> Wrap {
        let _preparing = self.codex.lock().await;

        codex::prepare(
            &self.fogde.join(".app").join("shell"),
            self.launcher.as_deref(),
        )
        .await
    }

    pub fn running_herdr(&self) -> Option<&Arc<Herdr>> {
        self.herdr.get()
    }

    pub async fn repos(&self) -> io::Result<PathBuf> {
        Ok(self
            .settings
            .repos()
            .await?
            .unwrap_or_else(|| self.fogde.join("repos")))
    }

    pub fn home(&self) -> &Path {
        &self.fogde
    }

    pub fn images(&self) -> PathBuf {
        self.fogde.join(".app").join("images")
    }

    pub fn worktrees(&self) -> PathBuf {
        self.fogde.join("worktrees")
    }

    pub fn turns(&self) -> PathBuf {
        self.fogde.join(".app").join("turns")
    }

    pub async fn set_repos(&self, path: PathBuf) -> io::Result<()> {
        fs::create_dir_all(&path).await?;
        self.settings.set_repos(path).await
    }
}

pub fn fogde() -> io::Result<PathBuf> {
    if let Some(home) = std::env::var_os("FOGDE_HOME") {
        return Ok(PathBuf::from(home));
    }

    std::env::home_dir()
        .map(|home| home.join("Fogde"))
        .ok_or_else(|| io::Error::other("could not find the home folder"))
}

pub fn app(fogde: &Path) -> io::Result<PathBuf> {
    let app = fogde.join(".app");
    std::fs::create_dir_all(&app)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o700))?;
    }

    Ok(app)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn keeps_the_app_folder_private() {
        let dir = tempfile::tempdir().unwrap();
        let fogde = dir.path().join("Fogde");
        std::fs::create_dir_all(fogde.join(".app")).unwrap();
        std::fs::set_permissions(fogde.join(".app"), std::fs::Permissions::from_mode(0o755))
            .unwrap();

        let app = app(&fogde).unwrap();

        let mode = std::fs::metadata(&app).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }
}
