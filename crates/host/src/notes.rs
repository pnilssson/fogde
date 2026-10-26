mod files;
mod watch;

use std::fmt::Write as _;
use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use protocol::folders::CreateFolderParams;
use protocol::notes::{
    CreateNote, CreateNoteFolder, DeleteNote, GetNotesFolder, ListNotes, Note, NoteList, NoteText,
    ReadNote, RenameNote, RenameNoteParams, SetNotesFolder, WriteNote, WriteNoteParams, Written,
};
use protocol::rpc::Failure;
use sha2::{Digest, Sha256};
use tokio::fs;

use crate::folders;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
use crate::shared::Shared;

pub use watch::{Notes, changes};

pub fn register(router: &mut Router) {
    router.add::<GetNotesFolder>();
    router.add::<SetNotesFolder>();
    router.add::<ListNotes>();
    router.add::<ReadNote>();
    router.add::<WriteNote>();
    router.add::<CreateNote>();
    router.add::<CreateNoteFolder>();
    router.add::<RenameNote>();
    router.add::<DeleteNote>();
}

impl Handler for GetNotesFolder {
    async fn handle(session: &Session, (): ()) -> Result<String, Failure> {
        let path = folder(&session.shared).await.map_err(failure)?;

        Ok(path.display().to_string())
    }
}

impl Handler for SetNotesFolder {
    async fn handle(session: &Session, path: String) -> Result<(), Failure> {
        let path = folders::absolute(&path).map_err(failure)?;
        folders::usable(&path).await.map_err(failure)?;
        fs::create_dir_all(&path).await.map_err(failure)?;

        session
            .shared
            .settings
            .set_notes(path)
            .await
            .map_err(failure)?;
        watch::changed(&session.shared);

        Ok(())
    }
}

impl Handler for ListNotes {
    async fn handle(session: &Session, (): ()) -> Result<NoteList, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        watch::watch(&session.shared, &root).await;

        tokio::task::spawn_blocking(move || list(&root))
            .await
            .map_err(failure)?
            .map_err(failure)
    }
}

impl Handler for ReadNote {
    async fn handle(session: &Session, path: String) -> Result<NoteText, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let path = folders::absolute(&path).map_err(failure)?;

        read(&root, &path).await.map_err(failure)
    }
}

impl Handler for WriteNote {
    async fn handle(session: &Session, params: WriteNoteParams) -> Result<Written, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let path = folders::absolute(&params.path).map_err(failure)?;

        write(&root, &path, &params.text, &params.version)
            .await
            .map_err(failure)
    }
}

impl Handler for CreateNote {
    async fn handle(session: &Session, params: CreateFolderParams) -> Result<String, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let parent = folders::absolute(&params.parent).map_err(failure)?;
        let path = files::note(&root, &parent, &params.name)
            .await
            .map_err(failure)?;

        Ok(path.display().to_string())
    }
}

impl Handler for CreateNoteFolder {
    async fn handle(session: &Session, params: CreateFolderParams) -> Result<String, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let parent = folders::absolute(&params.parent).map_err(failure)?;
        let path = files::folder(&root, &parent, &params.name)
            .await
            .map_err(failure)?;

        Ok(path.display().to_string())
    }
}

impl Handler for RenameNote {
    async fn handle(session: &Session, params: RenameNoteParams) -> Result<String, Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let path = folders::absolute(&params.path).map_err(failure)?;
        let renamed = files::rename(&root, &path, &params.name)
            .await
            .map_err(failure)?;
        if let Err(error) = crate::workspaces::follow(session, &path, &renamed).await {
            tracing::warn!(error = %error.message, "could not move the tiles showing a renamed note");
        }

        Ok(renamed.display().to_string())
    }
}

impl Handler for DeleteNote {
    async fn handle(session: &Session, path: String) -> Result<(), Failure> {
        let root = folder(&session.shared).await.map_err(failure)?;
        let path = folders::absolute(&path).map_err(failure)?;

        files::trash(&root, &path).await.map_err(failure)
    }
}

async fn folder(shared: &Shared) -> io::Result<PathBuf> {
    if let Some(path) = shared.settings.notes().await? {
        return Ok(path);
    }

    let path = shared.home().join("notes");
    fs::create_dir_all(&path).await?;

    Ok(path)
}

fn list(root: &Path) -> io::Result<NoteList> {
    let mut listing = NoteList {
        root: root.display().to_string(),
        folders: Vec::new(),
        notes: Vec::new(),
    };
    walk(root, &mut listing)?;
    listing.folders.sort_by_key(|path| path.to_lowercase());
    listing.notes.sort_by_key(|note| note.path.to_lowercase());

    Ok(listing)
}

fn walk(folder: &Path, listing: &mut NoteList) -> io::Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if folders::hidden(&entry.file_name().to_string_lossy(), &metadata) {
            continue;
        }

        let path = entry.path();
        if metadata.is_dir() {
            listing.folders.push(path.display().to_string());
            let _ = walk(&path, listing);
        } else if metadata.is_file() && markdown(&path) {
            listing.notes.push(note(&path, folder, &metadata));
        }
    }

    Ok(())
}

fn note(path: &Path, folder: &Path, metadata: &Metadata) -> Note {
    let modified = millis(metadata.modified()).unwrap_or_default();

    Note {
        name: path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: path.display().to_string(),
        folder: folder.display().to_string(),
        modified,
        created: millis(metadata.created()).unwrap_or(modified),
    }
}

async fn note_file(root: &Path, path: &Path) -> io::Result<PathBuf> {
    match files::entry(root, path).await? {
        files::Entry::Note(file) => Ok(file),
        files::Entry::Folder(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a note", path.display()),
        )),
    }
}

async fn read(root: &Path, path: &Path) -> io::Result<NoteText> {
    let bytes = fs::read(note_file(root, path).await?).await?;
    let version = version(&bytes);
    let text = String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} is not UTF-8 text", path.display()),
        )
    })?;

    Ok(NoteText { text, version })
}

async fn write(root: &Path, path: &Path, text: &str, expected: &str) -> io::Result<Written> {
    let file = note_file(root, path).await?;
    if version(&fs::read(&file).await?) != expected {
        return Ok(Written::Changed);
    }

    fs::write(&file, text).await?;

    Ok(Written::Saved {
        version: version(text.as_bytes()),
    })
}

fn version(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

fn markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn millis(time: io::Result<SystemTime>) -> Option<i64> {
    let since = time.ok()?.duration_since(UNIX_EPOCH).ok()?;

    since.as_millis().try_into().ok()
}

#[cfg(test)]
mod tests;
