use protocol::github::{Check, CheckState, MergeState};
use serde::Deserialize;

use crate::list::Nodes;

#[derive(Deserialize)]
pub(super) struct CommitNode {
    pub(super) commit: Commit,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Commit {
    pub(super) status_check_rollup: Option<Rollup>,
}

#[derive(Deserialize)]
pub(super) struct Rollup {
    pub(super) contexts: Nodes<Context>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Context {
    name: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    details_url: Option<String>,
    check_suite: Option<Suite>,
    #[serde(rename = "context")]
    label: Option<String>,
    state: Option<String>,
    target_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Suite {
    workflow_run: Option<Run>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Run {
    database_id: Option<u64>,
}

pub(super) fn check(context: Context) -> Option<Check> {
    if let Some(name) = context.name {
        let state = match (context.status.as_deref(), context.conclusion.as_deref()) {
            (Some("COMPLETED"), Some("SUCCESS")) => CheckState::Success,
            (Some("COMPLETED"), Some("SKIPPED")) => CheckState::Skipped,
            (Some("COMPLETED"), Some("NEUTRAL" | "STALE")) => CheckState::Neutral,
            (Some("COMPLETED"), _) => CheckState::Failure,
            _ => CheckState::Pending,
        };

        return Some(Check {
            name,
            state,
            url: context.details_url,
            run: context
                .check_suite
                .and_then(|suite| suite.workflow_run)
                .and_then(|run| run.database_id),
        });
    }

    let state = match context.state.as_deref()? {
        "SUCCESS" => CheckState::Success,
        "FAILURE" | "ERROR" => CheckState::Failure,
        _ => CheckState::Pending,
    };

    Some(Check {
        name: context.label?,
        state,
        url: context.target_url,
        run: None,
    })
}

pub(super) fn merge(status: &str) -> MergeState {
    match status {
        "CLEAN" | "HAS_HOOKS" => MergeState::Clean,
        "BEHIND" => MergeState::Behind,
        "BLOCKED" => MergeState::Blocked,
        "DIRTY" => MergeState::Conflicts,
        "DRAFT" => MergeState::Draft,
        "UNSTABLE" => MergeState::Unstable,
        _ => MergeState::Unknown,
    }
}
