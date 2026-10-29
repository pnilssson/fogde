use protocol::github::{ChecksState, ChecksSummary};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Rollup {
    state: String,
    contexts: Option<Contexts>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contexts {
    total_count: u32,
    #[serde(default)]
    check_run_counts_by_state: Vec<StateCount>,
    #[serde(default)]
    status_context_counts_by_state: Vec<StateCount>,
}

#[derive(Deserialize)]
struct StateCount {
    state: String,
    count: u32,
}

pub(crate) fn summary(rollup: Rollup) -> Option<ChecksSummary> {
    let state = checks(&rollup.state)?;
    let Some(contexts) = rollup.contexts else {
        return Some(ChecksSummary {
            state,
            passed: 0,
            failed: 0,
            total: 0,
        });
    };
    let counts = contexts
        .check_run_counts_by_state
        .iter()
        .chain(&contexts.status_context_counts_by_state);
    let (mut passed, mut failed, mut skipped) = (0, 0, 0);
    for count in counts {
        match count.state.as_str() {
            "SUCCESS" | "NEUTRAL" => passed += count.count,
            "SKIPPED" => skipped += count.count,
            "FAILURE" | "ERROR" | "TIMED_OUT" | "CANCELLED" | "STARTUP_FAILURE"
            | "ACTION_REQUIRED" => failed += count.count,
            _ => {}
        }
    }

    Some(ChecksSummary {
        state,
        passed,
        failed,
        total: contexts.total_count.saturating_sub(skipped),
    })
}

fn checks(state: &str) -> Option<ChecksState> {
    match state {
        "SUCCESS" => Some(ChecksState::Success),
        "FAILURE" | "ERROR" => Some(ChecksState::Failure),
        "PENDING" | "EXPECTED" => Some(ChecksState::Pending),
        _ => None,
    }
}
