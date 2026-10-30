use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;
use crate::repos::Repo;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Review,
    Mine,
    Assigned,
    Other,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ListParams {
    pub repo: Repo,
    pub filter: Filter,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ItemParams {
    pub repo: Repo,
    pub number: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Listed {
    pub number: u32,
    pub title: String,
    pub url: String,
    pub state: ItemState,
    pub author: Option<String>,
    pub labels: Vec<Label>,
    pub comments: u32,
    pub updated: String,
    pub section: Section,
    pub pull: Option<PullSummary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ItemState {
    Open,
    Closed,
    Merged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PullSummary {
    pub draft: bool,
    pub head: String,
    pub sha: String,
    pub owner: Option<String>,
    pub base: String,
    pub review: Option<ReviewState>,
    pub checks: Option<ChecksSummary>,
    pub added: u32,
    pub removed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ChecksSummary {
    pub state: ChecksState,
    pub passed: u32,
    pub failed: u32,
    pub total: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ReviewState {
    Approved,
    Changes,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ChecksState {
    Pending,
    Success,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Detail {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub url: String,
    pub state: ItemState,
    pub author: Option<String>,
    pub created: String,
    pub body: String,
    pub html: String,
    pub labels: Vec<Label>,
    pub assignees: Vec<String>,
    pub editable: bool,
    pub comments: Vec<Comment>,
    pub linked: Vec<Linked>,
    pub pull: Option<PullDetail>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Comment {
    pub id: String,
    pub author: Option<String>,
    pub html: String,
    pub created: String,
    pub url: String,
    pub verdict: Option<Verdict>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Approved,
    Changes,
    Commented,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Linked {
    pub number: u32,
    pub title: String,
    pub url: String,
    pub state: ItemState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PullDetail {
    pub draft: bool,
    pub head: String,
    pub owner: Option<String>,
    pub sha: String,
    pub base: String,
    pub cross: bool,
    pub review: Option<ReviewState>,
    pub merge: MergeState,
    pub methods: Vec<MergeMethod>,
    pub automerge: bool,
    pub auto: Option<MergeMethod>,
    pub reviewers: Vec<String>,
    pub checks: Vec<Check>,
    pub threads: Vec<Thread>,
    pub files: Vec<FileStat>,
    pub added: u32,
    pub removed: u32,
    pub changed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum MergeState {
    Clean,
    Behind,
    Blocked,
    Conflicts,
    Draft,
    Unstable,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Check {
    pub name: String,
    pub state: CheckState,
    pub url: Option<String>,
    #[ts(type = "number | null")]
    pub run: Option<u64>,
    #[ts(type = "number | null")]
    pub job: Option<u64>,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum CheckState {
    Pending,
    Success,
    Failure,
    Skipped,
    Neutral,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Thread {
    pub path: String,
    pub line: Option<u32>,
    pub outdated: bool,
    pub comments: Vec<Comment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
pub struct LogParams {
    pub repo: Repo,
    #[ts(type = "number")]
    pub job: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Log {
    pub step: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FileStat {
    pub path: String,
    pub added: u32,
    pub removed: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct RepoOptions {
    pub labels: Vec<Label>,
    pub people: Vec<String>,
    pub branch: Option<String>,
}

pub struct ListPulls;

impl Method for ListPulls {
    const NAME: &'static str = "github.pulls";
    type Params = ListParams;
    type Result = Vec<Listed>;
}

pub struct ListIssues;

impl Method for ListIssues {
    const NAME: &'static str = "github.issues";
    type Params = ListParams;
    type Result = Vec<Listed>;
}

pub struct GetPull;

impl Method for GetPull {
    const NAME: &'static str = "github.pull";
    type Params = ItemParams;
    type Result = Detail;
}

pub struct GetIssue;

impl Method for GetIssue {
    const NAME: &'static str = "github.issue";
    type Params = ItemParams;
    type Result = Detail;
}

pub struct GetOptions;

impl Method for GetOptions {
    const NAME: &'static str = "github.options";
    type Params = Repo;
    type Result = RepoOptions;
}

pub struct GetLog;

impl Method for GetLog {
    const NAME: &'static str = "github.log";
    type Params = LogParams;
    type Result = Log;
}
