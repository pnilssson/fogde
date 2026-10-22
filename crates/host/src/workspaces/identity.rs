use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use file_id::FileId;
use serde::Serialize;
use tokio::fs;

#[derive(Serialize)]
struct Identity {
    id: FileId,
    created: u128,
}

pub async fn of(folder: &Path) -> Option<String> {
    let folder = folder.to_owned();

    tokio::task::spawn_blocking(move || {
        let created = std::fs::metadata(&folder)
            .ok()?
            .created()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let id = file_id::get_file_id(&folder).ok()?;

        serde_json::to_string(&Identity { id, created }).ok()
    })
    .await
    .ok()
    .flatten()
}

#[derive(PartialEq)]
struct Listing {
    changed: SystemTime,
    names: Vec<OsString>,
}

impl Listing {
    async fn of(folder: &Path) -> Option<Self> {
        let changed = fs::metadata(folder).await.ok()?.modified().ok()?;
        let mut entries = fs::read_dir(folder).await.ok()?;
        let mut names = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            names.push(entry.file_name());
        }
        names.sort();

        Some(Self { changed, names })
    }
}

#[derive(Default)]
pub struct Searches {
    missed: Mutex<HashMap<String, Listing>>,
}

impl Searches {
    pub async fn find(&self, workspace: &str, folder: &Path, identity: &str) -> Option<PathBuf> {
        let parent = folder.parent()?;
        let listing = Listing::of(parent).await?;
        if self.missed().get(workspace) == Some(&listing) {
            return None;
        }

        let found = find(parent, identity).await;
        if found.is_none() {
            self.missed().insert(workspace.to_owned(), listing);
        }

        found
    }

    fn missed(&self) -> std::sync::MutexGuard<'_, HashMap<String, Listing>> {
        self.missed.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

async fn find(parent: &Path, identity: &str) -> Option<PathBuf> {
    let mut entries = fs::read_dir(parent).await.ok()?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        if !entry.file_type().await.is_ok_and(|kind| kind.is_dir()) {
            continue;
        }

        let path = entry.path();
        if of(&path).await.as_deref() == Some(identity) {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn finds_a_folder_renamed_beside_it() {
        let dir = tempfile::tempdir().unwrap();
        let before = dir.path().join("app");
        let after = dir.path().join("app-renamed");
        fs::create_dir(&before).await.unwrap();
        fs::create_dir(dir.path().join("other")).await.unwrap();
        let identity = of(&before).await.unwrap();

        fs::rename(&before, &after).await.unwrap();

        assert_eq!(
            Searches::default().find("1", &before, &identity).await,
            Some(after)
        );
    }

    #[tokio::test]
    async fn searches_again_only_once_the_parent_changes() {
        let dir = tempfile::tempdir().unwrap();
        let away = tempfile::tempdir().unwrap();
        let before = dir.path().join("app");
        let moved = away.path().join("app");
        fs::create_dir(&before).await.unwrap();
        let identity = of(&before).await.unwrap();
        let searches = Searches::default();

        fs::rename(&before, &moved).await.unwrap();
        assert_eq!(searches.find("1", &before, &identity).await, None);
        assert_eq!(searches.missed().len(), 1);

        fs::rename(&moved, dir.path().join("app-back"))
            .await
            .unwrap();
        assert_eq!(
            searches.find("1", &before, &identity).await,
            Some(dir.path().join("app-back"))
        );
    }

    #[tokio::test]
    async fn searches_again_when_the_parent_changes_within_one_clock_tick() {
        let dir = tempfile::tempdir().unwrap();
        let away = tempfile::tempdir().unwrap();
        let before = dir.path().join("app");
        let moved = away.path().join("app");
        fs::create_dir(&before).await.unwrap();
        let identity = of(&before).await.unwrap();
        let searches = Searches::default();

        fs::rename(&before, &moved).await.unwrap();
        let changed = filetime::FileTime::from_last_modification_time(
            &std::fs::metadata(dir.path()).unwrap(),
        );
        assert_eq!(searches.find("1", &before, &identity).await, None);

        fs::rename(&moved, dir.path().join("app-back"))
            .await
            .unwrap();
        filetime::set_file_mtime(dir.path(), changed).unwrap();
        assert_eq!(
            searches.find("1", &before, &identity).await,
            Some(dir.path().join("app-back"))
        );
    }

    #[tokio::test]
    async fn a_new_folder_in_its_place_is_another_folder() {
        let dir = tempfile::tempdir().unwrap();
        let before = dir.path().join("app");
        fs::create_dir(&before).await.unwrap();
        let identity = of(&before).await.unwrap();

        fs::remove_dir(&before).await.unwrap();
        fs::create_dir(dir.path().join("other")).await.unwrap();

        assert_eq!(
            Searches::default().find("1", &before, &identity).await,
            None
        );
    }
}
