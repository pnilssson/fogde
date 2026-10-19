use std::io;
use std::path::{Path, PathBuf};

use protocol::theme::Theme;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Default, Deserialize, Serialize)]
struct Values {
    #[serde(default)]
    onboarded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repos: Option<PathBuf>,
    #[serde(default)]
    theme: Theme,
    #[serde(default)]
    computer: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    servers: Vec<Server>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Server {
    pub id: String,
    pub target: String,
    pub triple: String,
}

pub struct Settings {
    path: PathBuf,
    writing: Mutex<()>,
}

impl Settings {
    pub fn new(app: &Path) -> Self {
        Self {
            path: app.join("settings.json"),
            writing: Mutex::new(()),
        }
    }

    pub async fn onboarded(&self) -> io::Result<bool> {
        Ok(self.load().await?.onboarded)
    }

    pub async fn finish_onboarding(&self) -> io::Result<()> {
        self.update(|values| values.onboarded = true).await
    }

    pub async fn repos(&self) -> io::Result<Option<PathBuf>> {
        Ok(self.load().await?.repos)
    }

    pub async fn set_repos(&self, path: PathBuf) -> io::Result<()> {
        self.update(|values| values.repos = Some(path)).await
    }

    pub async fn theme(&self) -> io::Result<Theme> {
        Ok(self.load().await?.theme)
    }

    pub async fn set_theme(&self, theme: Theme) -> io::Result<()> {
        self.update(|values| values.theme = theme).await
    }

    pub async fn computer(&self) -> io::Result<bool> {
        Ok(self.load().await?.computer)
    }

    pub async fn set_computer(&self, used: bool) -> io::Result<()> {
        self.update(|values| values.computer = used).await
    }

    pub async fn servers(&self) -> io::Result<Vec<Server>> {
        Ok(self.load().await?.servers)
    }

    pub async fn server(&self, id: &str) -> io::Result<Option<Server>> {
        Ok(self
            .load()
            .await?
            .servers
            .into_iter()
            .find(|server| server.id == id))
    }

    pub async fn add_server(&self, server: Server) -> io::Result<()> {
        self.update(|values| {
            values
                .servers
                .retain(|known| known.id != server.id && known.target != server.target);
            values.servers.push(server);
        })
        .await
    }

    pub async fn remove_server(&self, id: &str) -> io::Result<()> {
        self.update(|values| values.servers.retain(|server| server.id != id))
            .await
    }

    async fn load(&self) -> io::Result<Values> {
        match fs::read(&self.path).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Values::default()),
            Err(error) => Err(error),
        }
    }

    async fn update(&self, change: impl FnOnce(&mut Values)) -> io::Result<()> {
        let _writing = self.writing.lock().await;
        let mut values = self.load().await?;
        change(&mut values);

        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).await?;
        }
        let partial = self.path.with_extension("json.partial");
        let json = serde_json::to_vec_pretty(&values).map_err(io::Error::other)?;
        fs::write(&partial, json).await?;
        fs::rename(&partial, &self.path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(id: &str, target: &str) -> Server {
        Server {
            id: id.to_owned(),
            target: target.to_owned(),
            triple: "x86_64-unknown-linux-musl".to_owned(),
        }
    }

    #[tokio::test]
    async fn remembers_what_was_saved() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let settings = Settings::new(&app);

        assert!(!settings.onboarded().await.unwrap());
        assert_eq!(settings.repos().await.unwrap(), None);
        assert_eq!(settings.theme().await.unwrap(), Theme::System);
        assert!(!settings.computer().await.unwrap());

        settings.set_repos(app.join("repos")).await.unwrap();
        settings.finish_onboarding().await.unwrap();
        settings.set_theme(Theme::Dark).await.unwrap();
        settings.set_computer(true).await.unwrap();
        settings.add_server(server("a1", "my-vps")).await.unwrap();

        let reopened = Settings::new(&app);
        assert!(reopened.onboarded().await.unwrap());
        assert_eq!(reopened.repos().await.unwrap(), Some(app.join("repos")));
        assert_eq!(reopened.theme().await.unwrap(), Theme::Dark);
        assert!(reopened.computer().await.unwrap());
        assert_eq!(
            reopened.server("a1").await.unwrap(),
            Some(server("a1", "my-vps"))
        );
    }

    #[tokio::test]
    async fn keeps_one_entry_per_server() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings::new(dir.path());

        settings.add_server(server("a1", "my-vps")).await.unwrap();
        settings.add_server(server("b2", "work")).await.unwrap();
        settings
            .add_server(server("a1", "root@1.2.3.4"))
            .await
            .unwrap();
        settings.add_server(server("c3", "work")).await.unwrap();

        assert_eq!(
            settings.servers().await.unwrap(),
            [server("a1", "root@1.2.3.4"), server("c3", "work")]
        );

        settings.remove_server("a1").await.unwrap();

        assert_eq!(settings.servers().await.unwrap(), [server("c3", "work")]);
    }
}
