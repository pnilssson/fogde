use serde::Serialize;
use ts_rs::TS;

use crate::Method;

#[derive(Clone, Debug, Serialize, TS)]
pub struct Folder {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct Folders {
    pub path: String,
    pub parent: Option<String>,
    pub folders: Vec<Folder>,
}

pub struct ListFolders;

impl Method for ListFolders {
    const NAME: &'static str = "folders.list";
    type Params = String;
    type Result = Folders;
}
