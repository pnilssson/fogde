use serde::{Deserialize, Serialize};

use crate::Method;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Folder {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Folders {
    pub path: String,
    pub parent: Option<String>,
    pub folders: Vec<Folder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateFolderParams {
    pub parent: String,
    pub name: String,
}

pub struct ListFolders;

impl Method for ListFolders {
    const NAME: &'static str = "folders.list";
    type Params = String;
    type Result = Folders;
}

pub struct CreateFolder;

impl Method for CreateFolder {
    const NAME: &'static str = "folders.create";
    type Params = CreateFolderParams;
    type Result = Folder;
}

pub struct RemoveFolder;

impl Method for RemoveFolder {
    const NAME: &'static str = "folders.remove";
    type Params = String;
    type Result = ();
}
