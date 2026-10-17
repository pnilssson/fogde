use std::io;
use std::path::PathBuf;

use github::Gh;
use tokio::sync::OnceCell;

pub struct Shared {
    pub token: String,
    tools: PathBuf,
    gh: OnceCell<Gh>,
}

impl Shared {
    pub fn new(token: String) -> io::Result<Self> {
        let home = std::env::home_dir()
            .ok_or_else(|| io::Error::other("could not find the home folder"))?;

        Ok(Self {
            token,
            tools: home.join("Fogde").join(".app").join("tools"),
            gh: OnceCell::new(),
        })
    }

    pub async fn gh(&self) -> Result<&Gh, github::Error> {
        self.gh.get_or_try_init(|| Gh::locate(&self.tools)).await
    }
}
