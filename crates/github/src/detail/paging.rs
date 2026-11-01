use super::checks::Context;
use super::{ChecksRepo, Page, RawItem, Reply, ThreadsRepo};

pub(super) fn add_threads(raw: &mut RawItem, reply: Reply<ThreadsRepo>) {
    let page = reply
        .repository
        .and_then(|repo| repo.pull_request)
        .and_then(|pull| pull.review_threads);
    let Some(threads) = raw.review_threads.as_mut() else {
        return;
    };

    match page {
        Some(page) => threads.extend(page),
        None => threads.page_info = None,
    }
}

pub(super) fn add_checks(raw: &mut RawItem, reply: Reply<ChecksRepo>) {
    let page = reply
        .repository
        .and_then(|repo| repo.object)
        .and_then(|commit| commit.status_check_rollup)
        .map(|rollup| rollup.contexts);
    let Some(checks) = contexts(raw) else {
        return;
    };

    match page {
        Some(page) => checks.extend(page),
        None => checks.page_info = None,
    }
}

pub(super) fn contexts(raw: &mut RawItem) -> Option<&mut Page<Context>> {
    let node = raw.commits.as_mut()?.nodes.first_mut()?.as_mut()?;

    node.commit
        .status_check_rollup
        .as_mut()
        .map(|rollup| &mut rollup.contexts)
}
