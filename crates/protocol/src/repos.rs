use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

pub struct GetReposFolder;

impl Method for GetReposFolder {
    const NAME: &'static str = "repos.folder";
    type Params = ();
    type Result = String;
}

pub struct CheckReposFolder;

impl Method for CheckReposFolder {
    const NAME: &'static str = "repos.checkFolder";
    type Params = String;
    type Result = ();
}

pub struct SetReposFolder;

impl Method for SetReposFolder {
    const NAME: &'static str = "repos.setFolder";
    type Params = String;
    type Result = ();
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
pub struct Repo {
    pub host: String,
    pub owner: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct RemoteRepo {
    pub host: String,
    pub owner: String,
    pub name: String,
    pub cloned: bool,
}

pub struct ListRepos;

impl Method for ListRepos {
    const NAME: &'static str = "repos.list";
    type Params = String;
    type Result = Vec<RemoteRepo>;
}

pub struct CloneRepo;

impl Method for CloneRepo {
    const NAME: &'static str = "repos.clone";
    type Params = Repo;
    type Result = ();
}
