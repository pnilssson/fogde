use std::ffi::OsStr;
use std::fs::Metadata;
use std::io;
use std::path::{Component, Path, PathBuf};

use protocol::folders::{
    CreateFolder, CreateFolderParams, Folder, Folders, ListFolders, RemoveFolder,
};
use protocol::rpc::Failure;
use tokio::fs;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<ListFolders>();
    router.add::<CreateFolder>();
    router.add::<RemoveFolder>();
}

impl Handler for ListFolders {
    async fn handle(_: &Session, path: String) -> Result<Folders, Failure> {
        list(&absolute(&path).map_err(failure)?)
            .await
            .map_err(failure)
    }
}

impl Handler for CreateFolder {
    async fn handle(_: &Session, params: CreateFolderParams) -> Result<Folder, Failure> {
        let parent = absolute(&params.parent).map_err(failure)?;
        let name = params.name.trim();
        let path = create(&parent, name).await.map_err(failure)?;

        Ok(Folder {
            name: name.to_owned(),
            path: path.display().to_string(),
        })
    }
}

impl Handler for RemoveFolder {
    async fn handle(_: &Session, path: String) -> Result<(), Failure> {
        remove(&absolute(&path).map_err(failure)?)
            .await
            .map_err(failure)
    }
}

async fn list(path: &Path) -> io::Result<Folders> {
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

pub async fn directory(path: &Path) -> io::Result<()> {
    if fs::metadata(path).await.is_ok_and(|m| m.is_dir()) {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} is not a folder", path.display()),
    ))
}

pub async fn create(parent: &Path, name: &str) -> io::Result<PathBuf> {
    let mut parts = Path::new(name).components();
    let (Some(Component::Normal(_)), None) = (parts.next(), parts.next()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("\"{name}\" can't be a folder name"),
        ));
    };

    let path = parent.join(name);
    fs::create_dir_all(&path).await?;

    Ok(path)
}

pub async fn remove(path: &Path) -> io::Result<()> {
    fs::remove_dir(path).await
}

pub fn same(a: &Path, b: &Path) -> bool {
    inside(a, b) && inside(b, a)
}

pub fn inside(path: &Path, root: &Path) -> bool {
    let mut parts = path.components();

    root.components().all(|part| {
        parts
            .next()
            .is_some_and(|next| same_part(next.as_os_str(), part.as_os_str()))
    })
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

#[cfg(windows)]
fn same_part(a: &OsStr, b: &OsStr) -> bool {
    a.to_string_lossy()
        .eq_ignore_ascii_case(&b.to_string_lossy())
}

#[cfg(not(windows))]
fn same_part(a: &OsStr, b: &OsStr) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_visible_folders_from_the_nearest_existing_one() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
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
    }

    #[tokio::test]
    async fn accepts_folders_to_create_but_not_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("notes.txt"), "").await.unwrap();

        assert!(usable(root).await.is_ok());
        assert!(usable(&root.join("repos")).await.is_ok());
        assert!(usable(&root.join("notes.txt")).await.is_err());
    }

    #[tokio::test]
    async fn creates_one_named_folder_inside_its_parent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let created = create(root, "app").await.unwrap();
        let again = create(root, "app").await.unwrap();

        assert_eq!(created, root.join("app"));
        assert_eq!(again, created);
        assert!(created.is_dir());
        for name in ["", ".", "..", "../app", "app/src", "/app"] {
            assert!(create(root, name).await.is_err(), "{name}");
        }
        assert!(!root.parent().unwrap().join("app").exists());
    }

    #[tokio::test]
    async fn removes_only_empty_folders() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let empty = create(root, "empty").await.unwrap();
        let full = create(root, "full").await.unwrap();
        fs::write(full.join(".keep"), "").await.unwrap();

        remove(&empty).await.unwrap();
        let refused = remove(&full).await.unwrap_err();

        assert!(!empty.exists());
        assert_eq!(refused.kind(), io::ErrorKind::DirectoryNotEmpty);
        assert!(full.join(".keep").exists());
        assert!(remove(&root.join("notes.txt")).await.is_err());
    }

    #[test]
    fn compares_folders_by_their_parts() {
        let repo = std::env::temp_dir().join("app");

        assert!(inside(&repo.join("src"), &repo));
        assert!(inside(&repo, &repo));
        assert!(!inside(&repo, &repo.join("src")));
        assert!(!inside(&std::env::temp_dir().join("app-old"), &repo));
    }

    #[cfg(windows)]
    #[test]
    fn ignores_case_and_separators_on_windows() {
        assert!(inside(
            Path::new(r"c:\users\dev\App\src\"),
            Path::new("C:/Users/dev/app")
        ));
    }

    #[test]
    fn requires_a_full_path() {
        assert!(absolute("repos").is_err());
        assert!(absolute(&std::env::temp_dir().display().to_string()).is_ok());
    }
}
