use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use github::Gh;
use herdr_client::Herdr;
use remote::{Binaries, Tunnel};
use tokio::fs;
use tokio::sync::{Mutex, OnceCell};

use crate::db::Db;
use crate::settings::Settings;
use crate::terminals::Tracker;

pub struct Shared {
    pub settings: Settings,
    pub db: Db,
    pub binaries: Option<Binaries>,
    pub tunnels: Mutex<HashMap<String, Tunnel>>,
    pub placing: Mutex<()>,
    pub tracker: Arc<Tracker>,
    fogde: PathBuf,
    tools: PathBuf,
    gh: OnceCell<Gh>,
    herdr: OnceCell<Herdr>,
}

impl Shared {
    pub fn new(binaries: Option<Binaries>) -> io::Result<Self> {
        let fogde = fogde()?;
        let app = fogde.join(".app");

        Ok(Self {
            settings: Settings::new(&app),
            db: Db::new(app.join("fogde.db")),
            binaries,
            tunnels: Mutex::default(),
            placing: Mutex::default(),
            tracker: Arc::default(),
            tools: app.join("tools"),
            fogde,
            gh: OnceCell::new(),
            herdr: OnceCell::new(),
        })
    }

    pub async fn gh(&self) -> Result<&Gh, github::Error> {
        self.gh.get_or_try_init(|| Gh::locate(&self.tools)).await
    }

    pub async fn herdr(&self) -> Result<&Herdr, herdr_client::Error> {
        let home = self.fogde.join(".app").join("herdr");

        self.herdr
            .get_or_try_init(|| async move {
                let herdr = Herdr::start(&self.tools, &home).await?;
                tokio::spawn(Arc::clone(&self.tracker).run(herdr.client()));

                Ok(herdr)
            })
            .await
    }

    pub async fn repos(&self) -> io::Result<PathBuf> {
        Ok(self
            .settings
            .repos()
            .await?
            .unwrap_or_else(|| self.fogde.join("repos")))
    }

    pub fn worktrees(&self) -> PathBuf {
        self.fogde.join("worktrees")
    }

    pub async fn set_repos(&self, path: PathBuf) -> io::Result<()> {
        fs::create_dir_all(&path).await?;
        self.settings.set_repos(path).await
    }
}

pub fn fogde() -> io::Result<PathBuf> {
    std::env::home_dir()
        .map(|home| home.join("Fogde"))
        .ok_or_else(|| io::Error::other("could not find the home folder"))
}
