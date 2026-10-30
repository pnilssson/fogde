mod actions;
mod items;

use serde::Serialize;
use ts_rs::TS;

use crate::Method;

pub use actions::{
    AddComment, AutoMergeParams, CheckoutPull, CheckoutPullParams, CommentParams, DraftParams,
    EditItem, EditParams, LabelsParams, MergeParams, MergePull, NewIssue, NewIssueParams, NewPull,
    NewPullParams, PeopleParams, RerunFailed, RerunParams, ReviewEvent, ReviewParams, SetAssignees,
    SetAutoMerge, SetDraft, SetLabels, SetReviewers, SetState, StateParams, SubmitReview,
    UpdateBranch,
};
pub use items::{
    Check, CheckState, ChecksState, ChecksSummary, Comment, Detail, FileStat, Filter, GetIssue,
    GetLog, GetOptions, GetPull, ItemParams, ItemState, Label, Linked, ListIssues, ListParams,
    ListPulls, Listed, Log, LogParams, MergeMethod, MergeState, PullDetail, PullSummary,
    RepoOptions, ReviewState, Section, Thread, Verdict,
};

#[derive(Clone, Debug, Serialize, TS)]
pub struct Account {
    pub host: String,
    pub login: String,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct SignInCode {
    pub code: String,
    pub url: String,
}

pub struct GetAccounts;

impl Method for GetAccounts {
    const NAME: &'static str = "github.accounts";
    type Params = ();
    type Result = Vec<Account>;
}

pub struct StartSignIn;

impl Method for StartSignIn {
    const NAME: &'static str = "github.startSignIn";
    type Params = String;
    type Result = SignInCode;
}

pub struct FinishSignIn;

impl Method for FinishSignIn {
    const NAME: &'static str = "github.finishSignIn";
    type Params = ();
    type Result = ();
}

pub struct CancelSignIn;

impl Method for CancelSignIn {
    const NAME: &'static str = "github.cancelSignIn";
    type Params = ();
    type Result = ();
}
