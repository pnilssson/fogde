use std::io;
use std::path::PathBuf;

use github::Gh;
use tokio::sync::OnceCell;

use crate::settings::Settings;

pub struct Shared {
    pub token: String,
    pub settings: Settings,
    tools: PathBuf,
    gh: OnceCell<Gh>,
}

impl Shared {
    pub fn new(token: String) -> io::Result<Self> {
        let home = std::env::home_dir()
            .ok_or_else(|| io::Error::other("could not find the home folder"))?;
        let app = home.join("Fogde").join(".app");

        Ok(Self {
            token,
            settings: Settings::new(&app),
            tools: app.join("tools"),
            gh: OnceCell::new(),
        })
    }

    pub async fn gh(&self) -> Result<&Gh, github::Error> {
        self.gh.get_or_try_init(|| Gh::locate(&self.tools)).await
    }
}
