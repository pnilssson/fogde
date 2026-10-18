use std::io;
use std::path::PathBuf;

use github::Gh;
use remote::{Binaries, Tunnel};
use tokio::fs;
use tokio::sync::{Mutex, OnceCell};

use crate::settings::Settings;

pub struct Shared {
    pub settings: Settings,
    pub binaries: Option<Binaries>,
    pub tunnel: Mutex<Option<Tunnel>>,
    fogde: PathBuf,
    tools: PathBuf,
    gh: OnceCell<Gh>,
}

impl Shared {
    pub fn new(binaries: Option<Binaries>) -> io::Result<Self> {
        let fogde = fogde()?;
        let app = fogde.join(".app");

        Ok(Self {
            settings: Settings::new(&app),
            binaries,
            tunnel: Mutex::new(None),
            tools: app.join("tools"),
            fogde,
            gh: OnceCell::new(),
        })
    }

    pub async fn gh(&self) -> Result<&Gh, github::Error> {
        self.gh.get_or_try_init(|| Gh::locate(&self.tools)).await
    }

    pub async fn repos(&self) -> io::Result<PathBuf> {
        Ok(self
            .settings
            .repos()
            .await?
            .unwrap_or_else(|| self.fogde.join("repos")))
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
