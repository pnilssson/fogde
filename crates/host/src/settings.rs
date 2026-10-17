use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Default, Deserialize, Serialize)]
struct Values {
    #[serde(default)]
    onboarded: bool,
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

    #[tokio::test]
    async fn remembers_finished_onboarding() {
        let app = std::env::temp_dir().join(format!("fogde-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&app).await;
        let settings = Settings::new(&app);

        assert!(!settings.onboarded().await.unwrap());

        settings.finish_onboarding().await.unwrap();

        assert!(Settings::new(&app).onboarded().await.unwrap());
        fs::remove_dir_all(&app).await.unwrap();
    }
}
