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
    let path = entry(parent, name)?;
    fs::create_dir_all(&path).await?;

    Ok(path)
}

pub fn entry(parent: &Path, name: &str) -> io::Result<PathBuf> {
    let mut parts = Path::new(name).components();
    let (Some(Component::Normal(_)), None) = (parts.next(), parts.next()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("\"{name}\" can't be a name"),
        ));
    };

    Ok(parent.join(name))
}

pub async fn remove(path: &Path) -> io::Result<()> {
    fs::remove_dir(path).await
}

pub fn same(a: &Path, b: &Path) -> bool {
    inside(a, b) && inside(b, a)
}

pub async fn identical(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a).await, fs::canonicalize(b).await) {
        (Ok(a), Ok(b)) => same(&a, &b),
        _ => same(a, b),
    }
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
pub fn hidden(name: &str, metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const HIDDEN: u32 = 0x2;

    name.starts_with('.') || metadata.file_attributes() & HIDDEN != 0
}

#[cfg(not(windows))]
pub fn hidden(name: &str, _metadata: &Metadata) -> bool {
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
mod tests;
