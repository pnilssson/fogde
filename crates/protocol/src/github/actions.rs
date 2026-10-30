use serde::Deserialize;
use ts_rs::TS;

use super::items::{ItemParams, MergeMethod};
use crate::Method;
use crate::repos::Repo;

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CommentParams {
    pub repo: Repo,
    pub number: u32,
    pub body: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum ReviewEvent {
    Approve,
    Changes,
    Comment,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ReviewParams {
    pub repo: Repo,
    pub number: u32,
    pub event: ReviewEvent,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct MergeParams {
    pub repo: Repo,
    pub number: u32,
    pub method: MergeMethod,
    pub sha: String,
    pub delete: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct DraftParams {
    pub repo: Repo,
    pub id: String,
    pub draft: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct AutoMergeParams {
    pub repo: Repo,
    pub id: String,
    pub method: Option<MergeMethod>,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct StateParams {
    pub repo: Repo,
    pub number: u32,
    pub pull: bool,
    pub open: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct EditParams {
    pub repo: Repo,
    pub number: u32,
    pub pull: bool,
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct LabelsParams {
    pub repo: Repo,
    pub number: u32,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct PeopleParams {
    pub repo: Repo,
    pub number: u32,
    pub add: Vec<String>,
    pub remove: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct RerunParams {
    pub repo: Repo,
    #[ts(type = "number")]
    pub run: u64,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct NewIssueParams {
    pub repo: Repo,
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct NewPullParams {
    pub path: String,
    pub repo: Repo,
    pub title: String,
    pub body: String,
    pub base: String,
    pub draft: bool,
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct CheckoutPullParams {
    pub path: String,
    pub repo: Repo,
    pub number: u32,
    pub worktree: bool,
}

pub struct AddComment;

impl Method for AddComment {
    const NAME: &'static str = "github.comment";
    type Params = CommentParams;
    type Result = ();
}

pub struct SubmitReview;

impl Method for SubmitReview {
    const NAME: &'static str = "github.review";
    type Params = ReviewParams;
    type Result = ();
}

pub struct MergePull;

impl Method for MergePull {
    const NAME: &'static str = "github.merge";
    type Params = MergeParams;
    type Result = ();
}

pub struct UpdateBranch;

impl Method for UpdateBranch {
    const NAME: &'static str = "github.updateBranch";
    type Params = ItemParams;
    type Result = ();
}

pub struct SetAutoMerge;

impl Method for SetAutoMerge {
    const NAME: &'static str = "github.setAutoMerge";
    type Params = AutoMergeParams;
    type Result = ();
}

pub struct SetDraft;

impl Method for SetDraft {
    const NAME: &'static str = "github.setDraft";
    type Params = DraftParams;
    type Result = ();
}

pub struct SetState;

impl Method for SetState {
    const NAME: &'static str = "github.setState";
    type Params = StateParams;
    type Result = ();
}

pub struct EditItem;

impl Method for EditItem {
    const NAME: &'static str = "github.edit";
    type Params = EditParams;
    type Result = ();
}

pub struct SetLabels;

impl Method for SetLabels {
    const NAME: &'static str = "github.setLabels";
    type Params = LabelsParams;
    type Result = ();
}

pub struct SetAssignees;

impl Method for SetAssignees {
    const NAME: &'static str = "github.setAssignees";
    type Params = PeopleParams;
    type Result = ();
}

pub struct SetReviewers;

impl Method for SetReviewers {
    const NAME: &'static str = "github.setReviewers";
    type Params = PeopleParams;
    type Result = ();
}

pub struct RerunFailed;

impl Method for RerunFailed {
    const NAME: &'static str = "github.rerun";
    type Params = RerunParams;
    type Result = ();
}

pub struct NewIssue;

impl Method for NewIssue {
    const NAME: &'static str = "github.newIssue";
    type Params = NewIssueParams;
    type Result = u32;
}

pub struct NewPull;

impl Method for NewPull {
    const NAME: &'static str = "github.newPull";
    type Params = NewPullParams;
    type Result = u32;
}

pub struct CheckoutPull;

impl Method for CheckoutPull {
    const NAME: &'static str = "github.checkoutPull";
    type Params = CheckoutPullParams;
    type Result = String;
}
