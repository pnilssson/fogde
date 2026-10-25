use std::io;
use std::path::{Path, PathBuf};

use protocol::accessibility::Accessibility;
use protocol::notifications::Notifications;
use protocol::stats::Stats;
use protocol::theme::{Material, Theme};
use protocol::usage::Shown;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Default, Deserialize, Serialize)]
struct Values {
    #[serde(default)]
    onboarded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repos: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    notes: Option<PathBuf>,
    #[serde(default)]
    theme: Theme,
    #[serde(default)]
    material: Material,
    #[serde(default)]
    notifications: Notifications,
    #[serde(default)]
    accessibility: Accessibility,
    #[serde(default)]
    stats: Stats,
    #[serde(default)]
    shown: Shown,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shell: Option<String>,
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

    pub async fn notes(&self) -> io::Result<Option<PathBuf>> {
        Ok(self.load().await?.notes)
    }

    pub async fn set_notes(&self, path: PathBuf) -> io::Result<()> {
        self.update(|values| values.notes = Some(path)).await
    }

    pub async fn theme(&self) -> io::Result<Theme> {
        Ok(self.load().await?.theme)
    }

    pub async fn set_theme(&self, theme: Theme) -> io::Result<()> {
        self.update(|values| values.theme = theme).await
    }

    pub async fn material(&self) -> io::Result<Material> {
        Ok(self.load().await?.material)
    }

    pub async fn set_material(&self, material: Material) -> io::Result<()> {
        self.update(|values| values.material = material).await
    }

    pub async fn notifications(&self) -> io::Result<Notifications> {
        Ok(self.load().await?.notifications)
    }

    pub async fn set_notifications(&self, notifications: Notifications) -> io::Result<()> {
        self.update(|values| values.notifications = notifications)
            .await
    }

    pub async fn accessibility(&self) -> io::Result<Accessibility> {
        Ok(self.load().await?.accessibility)
    }

    pub async fn set_accessibility(&self, accessibility: Accessibility) -> io::Result<()> {
        self.update(|values| values.accessibility = accessibility)
            .await
    }

    pub async fn stats(&self) -> io::Result<Stats> {
        Ok(self.load().await?.stats)
    }

    pub async fn set_stats(&self, stats: Stats) -> io::Result<()> {
        self.update(|values| values.stats = stats).await
    }

    pub async fn shown(&self) -> io::Result<Shown> {
        Ok(self.load().await?.shown)
    }

    pub async fn set_shown(&self, shown: Shown) -> io::Result<()> {
        self.update(|values| values.shown = shown).await
    }

    pub async fn shell(&self) -> io::Result<Option<String>> {
        Ok(self.load().await?.shell)
    }

    pub async fn set_shell(&self, shell: Option<String>) -> io::Result<()> {
        self.update(|values| values.shell = shell).await
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
    async fn remembers_what_was_saved() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let settings = Settings::new(&app);

        assert!(!settings.onboarded().await.unwrap());
        assert_eq!(settings.repos().await.unwrap(), None);
        assert_eq!(settings.notes().await.unwrap(), None);
        assert_eq!(settings.theme().await.unwrap(), Theme::System);
        assert_eq!(settings.material().await.unwrap(), Material::Translucent);
        assert_eq!(
            settings.accessibility().await.unwrap(),
            Accessibility::default()
        );
        assert_eq!(
            settings.notifications().await.unwrap(),
            Notifications::default()
        );

        settings.set_repos(app.join("repos")).await.unwrap();
        settings.set_notes(app.join("notes")).await.unwrap();
        settings.finish_onboarding().await.unwrap();
        settings.set_theme(Theme::Dark).await.unwrap();
        settings.set_material(Material::Solid).await.unwrap();
        let quiet = Notifications {
            enabled: true,
            sound: false,
        };
        settings.set_notifications(quiet).await.unwrap();
        let reader = Accessibility {
            screen_reader: true,
        };
        settings.set_accessibility(reader).await.unwrap();
        let quiet_codex = Shown {
            codex: false,
            ..Shown::default()
        };
        settings.set_shown(quiet_codex).await.unwrap();
        settings.set_shell(Some("zsh".into())).await.unwrap();

        let reopened = Settings::new(&app);
        assert!(reopened.onboarded().await.unwrap());
        assert_eq!(reopened.repos().await.unwrap(), Some(app.join("repos")));
        assert_eq!(reopened.notes().await.unwrap(), Some(app.join("notes")));
        assert_eq!(reopened.theme().await.unwrap(), Theme::Dark);
        assert_eq!(reopened.material().await.unwrap(), Material::Solid);
        assert_eq!(reopened.notifications().await.unwrap(), quiet);
        assert_eq!(reopened.accessibility().await.unwrap(), reader);
        assert_eq!(reopened.shown().await.unwrap(), quiet_codex);
        assert_eq!(reopened.shell().await.unwrap().as_deref(), Some("zsh"));
    }
}
