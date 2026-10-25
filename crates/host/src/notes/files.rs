use std::io;
use std::path::{Path, PathBuf};

use tokio::fs;

use super::markdown;
use crate::folders;

pub(super) enum Entry {
    Note(PathBuf),
    Folder(PathBuf),
}

pub(super) async fn note(root: &Path, parent: &Path, name: &str) -> io::Result<PathBuf> {
    let name = name.trim();
    let path = named(root, parent, &file_name(name)).await?;

    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|error| taken(error, "note", name))?;

    Ok(path)
}

pub(super) async fn rename(root: &Path, path: &Path, name: &str) -> io::Result<PathBuf> {
    let entry = entry(root, path).await?;
    let Some(parent) = path.parent() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} has no folder", path.display()),
        ));
    };
    let name = name.trim();
    let (real, kind, file) = match entry {
        Entry::Note(real) => (real, "note", file_name(name)),
        Entry::Folder(real) => (real, "folder", name.to_owned()),
    };
    let target = named(root, parent, &file).await?;
    let same = fs::canonicalize(&target)
        .await
        .is_ok_and(|existing| existing == real);
    if fs::try_exists(&target).await? && !same {
        return Err(taken(io::ErrorKind::AlreadyExists.into(), kind, name));
    }

    fs::rename(path, &target).await?;

    Ok(target)
}

pub(super) async fn trash(root: &Path, path: &Path) -> io::Result<()> {
    let (stem, extension) = match entry(root, path).await? {
        Entry::Note(_) => (path.file_stem(), ".md"),
        Entry::Folder(_) => (path.file_name(), ""),
    };
    let stem = stem
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let bin = root.join(".trash");
    fs::create_dir_all(&bin).await?;

    let mut target = bin.join(format!("{stem}{extension}"));
    let mut copy = 1;
    while fs::try_exists(&target).await? {
        copy += 1;
        target = bin.join(format!("{stem} {copy}{extension}"));
    }

    fs::rename(path, target).await
}

pub(super) async fn folder(root: &Path, parent: &Path, name: &str) -> io::Result<PathBuf> {
    let name = name.trim();
    let path = named(root, parent, name).await?;

    fs::create_dir(&path)
        .await
        .map_err(|error| taken(error, "folder", name))?;

    Ok(path)
}

pub(super) async fn entry(root: &Path, path: &Path) -> io::Result<Entry> {
    let root = fs::canonicalize(root).await?;
    let real = fs::canonicalize(path).await?;
    let metadata = fs::metadata(&real).await?;
    let visible = real.strip_prefix(&root).is_ok_and(|inner| {
        inner
            .components()
            .all(|part| !part.as_os_str().to_string_lossy().starts_with('.'))
    });

    if visible && path.file_name().is_some() && !folders::same(&real, &root) {
        if metadata.is_file() && markdown(&real) {
            return Ok(Entry::Note(real));
        }
        if metadata.is_dir() {
            return Ok(Entry::Folder(real));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!(
            "{} is not a note or folder in the notes folder",
            path.display()
        ),
    ))
}

fn file_name(name: &str) -> String {
    if markdown(Path::new(name)) {
        name.to_owned()
    } else {
        format!("{name}.md")
    }
}

async fn named(root: &Path, parent: &Path, name: &str) -> io::Result<PathBuf> {
    if name.starts_with('.') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Names starting with a dot are hidden from notes",
        ));
    }

    let root = fs::canonicalize(root).await?;
    let real = fs::canonicalize(parent).await?;
    if !folders::inside(&real, &root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{} is not in the notes folder", parent.display()),
        ));
    }

    folders::entry(parent, name)
}

fn taken(error: io::Error, kind: &str, name: &str) -> io::Error {
    if error.kind() == io::ErrorKind::AlreadyExists {
        return io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("A {kind} named \"{name}\" already exists here"),
        );
    }

    error
}

#[cfg(test)]
mod tests;
