use std::io;
use std::path::PathBuf;

use github::Gh;
use tokio::fs;
use tokio::sync::OnceCell;

use crate::settings::Settings;

pub struct Shared {
    pub token: String,
    pub settings: Settings,
    fogde: PathBuf,
    tools: PathBuf,
    gh: OnceCell<Gh>,
}

impl Shared {
    pub fn new(token: String) -> io::Result<Self> {
        let home = std::env::home_dir()
            .ok_or_else(|| io::Error::other("could not find the home folder"))?;
        let fogde = home.join("Fogde");
        let app = fogde.join(".app");

        Ok(Self {
            token,
            settings: Settings::new(&app),
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
