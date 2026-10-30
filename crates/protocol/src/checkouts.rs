use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::repos::Repo;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CheckoutStatus {
    pub path: String,
    pub repo: Option<Repo>,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<FileChange>,
    pub truncated: bool,
    pub owner: Option<String>,
    pub pull: Option<u32>,
    #[ts(type = "number | null")]
    pub fetched: Option<u64>,
    pub last: Option<LastCommit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct LastCommit {
    pub sha: String,
    pub subject: String,
    pub date: String,
    pub undoable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Branch {
    pub name: String,
    pub worktree: Option<String>,
    pub remote: bool,
    pub date: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FileChange {
    pub path: String,
    pub from: Option<String>,
    pub change: ChangeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CommitParams {
    pub path: String,
    pub paths: Vec<String>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum DiffScope {
    Uncommitted,
    Branch,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct DiffParams {
    pub path: String,
    pub scope: DiffScope,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Diff {
    pub base: Option<String>,
    pub files: Vec<FileDiff>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FileDiff {
    pub path: String,
    pub from: Option<String>,
    pub change: ChangeKind,
    pub added: u32,
    pub removed: u32,
    pub patch: Option<String>,
    pub binary: bool,
    pub large: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct UndoParams {
    pub path: String,
    pub sha: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CleanUpParams {
    pub path: String,
    pub sha: String,
    pub base: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct SwitchParams {
    pub path: String,
    pub branch: String,
}

pub struct GetCheckouts;

impl Method for GetCheckouts {
    const NAME: &'static str = "checkouts.status";
    type Params = Vec<String>;
    type Result = Vec<CheckoutStatus>;
}

pub struct ListBranches;

impl Method for ListBranches {
    const NAME: &'static str = "checkouts.branches";
    type Params = String;
    type Result = Vec<Branch>;
}

pub struct Commit;

impl Method for Commit {
    const NAME: &'static str = "checkouts.commit";
    type Params = CommitParams;
    type Result = ();
}

pub struct Push;

impl Method for Push {
    const NAME: &'static str = "checkouts.push";
    type Params = String;
    type Result = ();
}

pub struct Pull;

impl Method for Pull {
    const NAME: &'static str = "checkouts.pull";
    type Params = String;
    type Result = ();
}

pub struct Fetch;

impl Method for Fetch {
    const NAME: &'static str = "checkouts.fetch";
    type Params = String;
    type Result = ();
}

pub struct GetDiff;

impl Method for GetDiff {
    const NAME: &'static str = "checkouts.diff";
    type Params = DiffParams;
    type Result = Diff;
}

pub struct CleanUp;

impl Method for CleanUp {
    const NAME: &'static str = "checkouts.cleanUp";
    type Params = CleanUpParams;
    type Result = ();
}

pub struct Undo;

impl Method for Undo {
    const NAME: &'static str = "checkouts.undo";
    type Params = UndoParams;
    type Result = ();
}

pub struct CreateBranch;

impl Method for CreateBranch {
    const NAME: &'static str = "checkouts.createBranch";
    type Params = SwitchParams;
    type Result = ();
}

pub struct Switch;

impl Method for Switch {
    const NAME: &'static str = "checkouts.switch";
    type Params = SwitchParams;
    type Result = ();
}
