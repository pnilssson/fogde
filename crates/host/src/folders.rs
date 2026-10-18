use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};

use protocol::folders::{Folder, Folders};
use tokio::fs;

pub async fn list(path: &Path) -> io::Result<Folders> {
    let path = existing(path).await?;
    let mut entries = fs::read_dir(&path).await?;
    let mut folders = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(link) = entry.metadata().await else {
            continue;
        };
        if hidden(&name, &link) || !fs::metadata(entry.path()).await.is_ok_and(|m| m.is_dir()) {
            continue;
        }

        folders.push(Folder {
            name,
            path: entry.path().display().to_string(),
        });
    }
    folders.sort_by_key(|folder| folder.name.to_lowercase());

    Ok(Folders {
        path: path.display().to_string(),
        parent: path.parent().map(|parent| parent.display().to_string()),
        folders,
    })
}

pub async fn usable(path: &Path) -> io::Result<()> {
    if fs::metadata(path).await.is_ok_and(|m| !m.is_dir()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is a file", path.display()),
        ));
    }

    existing(path).await.map(|_| ())
}

pub fn absolute(path: &str) -> io::Result<PathBuf> {
    let path: PathBuf = Path::new(path.trim()).components().collect();
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a full path", path.display()),
        ));
    }

    Ok(path)
}

async fn existing(path: &Path) -> io::Result<PathBuf> {
    for candidate in path.ancestors() {
        if fs::metadata(candidate).await.is_ok_and(|m| m.is_dir()) {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Could not find {}", path.display()),
    ))
}

#[cfg(windows)]
fn hidden(name: &str, metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const HIDDEN: u32 = 0x2;

    name.starts_with('.') || metadata.file_attributes() & HIDDEN != 0
}

#[cfg(not(windows))]
fn hidden(name: &str, _metadata: &Metadata) -> bool {
    name.starts_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_visible_folders_from_the_nearest_existing_one() {
        let root = std::env::temp_dir().join(format!("fogde-folders-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root).await;
        for folder in ["beta", "Alpha", ".git"] {
            fs::create_dir_all(root.join(folder)).await.unwrap();
        }
        fs::write(root.join("notes.txt"), "").await.unwrap();

        let listing = list(&root.join("missing").join("deeper")).await.unwrap();

        assert_eq!(listing.path, root.display().to_string());
        assert_eq!(
            listing
                .folders
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["Alpha", "beta"]
        );
        assert_eq!(
            listing.folders[0].path,
            root.join("Alpha").display().to_string()
        );
        fs::remove_dir_all(&root).await.unwrap();
    }

    #[tokio::test]
    async fn accepts_folders_to_create_but_not_files() {
        let root = std::env::temp_dir().join(format!("fogde-usable-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root).await;
        fs::create_dir_all(&root).await.unwrap();
        fs::write(root.join("notes.txt"), "").await.unwrap();

        assert!(usable(&root).await.is_ok());
        assert!(usable(&root.join("repos")).await.is_ok());
        assert!(usable(&root.join("notes.txt")).await.is_err());
        fs::remove_dir_all(&root).await.unwrap();
    }

    #[test]
    fn requires_a_full_path() {
        assert!(absolute("repos").is_err());
        assert!(absolute(&std::env::temp_dir().display().to_string()).is_ok());
    }
}
