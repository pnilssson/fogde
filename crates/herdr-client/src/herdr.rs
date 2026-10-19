use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use serde::de::IgnoredAny;
use tokio::fs;

use crate::client::Empty;
use crate::install::{self, VERSION};
use crate::{Client, Error};

const CONFIG: &str = "onboarding = false

[update]
version_check = false

[ui.sound]
enabled = false
";

const PANE_ENV: [&str; 2] = ["XDG_CONFIG_HOME", "XDG_STATE_HOME"];

pub struct Herdr {
    program: PathBuf,
    home: PathBuf,
    env: BTreeMap<String, String>,
}

impl Herdr {
    pub async fn start(tools: &Path, home: &Path) -> Result<Self, Error> {
        let herdr = Self {
            program: install::locate(tools).await?,
            home: home.to_owned(),
            env: PANE_ENV
                .iter()
                .map(|&name| (name.to_owned(), std::env::var(name).unwrap_or_default()))
                .collect(),
        };
        herdr.configure().await?;

        let client = herdr.client();
        if let Ok(pong) = client.ping().await {
            let running = pong.version;
            if running == VERSION {
                return Ok(herdr);
            }

            tracing::info!(%running, "restarting Herdr on {VERSION}");
            client
                .request::<IgnoredAny>("server.stop", Empty {})
                .await?;
            herdr.wait(false).await?;
        }

        process::detached(&herdr.program)
            .arg("server")
            .env("XDG_CONFIG_HOME", herdr.config())
            .env("XDG_STATE_HOME", herdr.home.join("state"))
            .env_remove("HERDR_SOCKET_PATH")
            .env_remove("HERDR_SESSION")
            .spawn()
            .map_err(Error::Start)?;
        herdr.wait(true).await?;

        Ok(herdr)
    }

    pub fn client(&self) -> Client {
        Client::new(&self.socket())
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn socket(&self) -> PathBuf {
        self.config().join("herdr").join("herdr.sock")
    }

    pub fn pane_env(&self) -> &BTreeMap<String, String> {
        &self.env
    }

    fn config(&self) -> PathBuf {
        self.home.join("config")
    }

    async fn configure(&self) -> Result<(), Error> {
        let dir = self.config().join("herdr");
        let path = dir.join("config.toml");
        if fs::read_to_string(&path)
            .await
            .is_ok_and(|current| current == CONFIG)
        {
            return Ok(());
        }

        fs::create_dir_all(&dir).await?;
        fs::write(&path, CONFIG).await?;

        Ok(())
    }

    async fn wait(&self, running: bool) -> Result<(), Error> {
        for _ in 0..100 {
            if self.client().ping().await.is_ok() == running {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(Error::Timeout)
    }
}

impl Client {
    async fn ping(&self) -> Result<Pong, Error> {
        self.request("ping", Empty {}).await
    }
}

#[derive(Deserialize)]
struct Pong {
    version: String,
}
