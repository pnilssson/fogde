use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::folders::CreateFolderParams;
use crate::{Event, Method};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct NoteList {
    pub root: String,
    pub folders: Vec<String>,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Note {
    pub name: String,
    pub path: String,
    pub folder: String,
    #[ts(type = "number")]
    pub modified: i64,
    #[ts(type = "number")]
    pub created: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct NoteText {
    pub text: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct WriteNoteParams {
    pub path: String,
    pub text: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Written {
    Saved { version: String },
    Changed,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct RenameNoteParams {
    pub path: String,
    pub name: String,
}

pub struct GetNotesFolder;

impl Method for GetNotesFolder {
    const NAME: &'static str = "notes.folder";
    type Params = ();
    type Result = String;
}

pub struct SetNotesFolder;

impl Method for SetNotesFolder {
    const NAME: &'static str = "notes.setFolder";
    type Params = String;
    type Result = ();
}

pub struct ListNotes;

impl Method for ListNotes {
    const NAME: &'static str = "notes.list";
    type Params = ();
    type Result = NoteList;
}

pub struct ReadNote;

impl Method for ReadNote {
    const NAME: &'static str = "notes.read";
    type Params = String;
    type Result = NoteText;
}

pub struct WriteNote;

impl Method for WriteNote {
    const NAME: &'static str = "notes.write";
    type Params = WriteNoteParams;
    type Result = Written;
}

pub struct CreateNote;

impl Method for CreateNote {
    const NAME: &'static str = "notes.createNote";
    type Params = CreateFolderParams;
    type Result = String;
}

pub struct CreateNoteFolder;

impl Method for CreateNoteFolder {
    const NAME: &'static str = "notes.createFolder";
    type Params = CreateFolderParams;
    type Result = String;
}

pub struct RenameNote;

impl Method for RenameNote {
    const NAME: &'static str = "notes.rename";
    type Params = RenameNoteParams;
    type Result = String;
}

pub struct DeleteNote;

impl Method for DeleteNote {
    const NAME: &'static str = "notes.delete";
    type Params = String;
    type Result = ();
}

pub struct NotesEvent;

impl Event for NotesEvent {
    const NAME: &'static str = "notes.changed";
    type Params = ();
}
