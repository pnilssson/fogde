use protocol::github::{Detail, MergeMethod};
use protocol::repos::Repo;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::Error;
use crate::api::checked;
use crate::gh::Gh;
use crate::list::{Login, Nodes, RawLabel};

mod build;
mod checks;
mod conversation;
mod options;
mod paging;

use build::detail;
use checks::{AutoMerge, Commit, CommitNode, merge, method};
use conversation::{RawComment, RawThread, comment, thread};
use paging::{add_checks, add_threads, contexts};

const PULL: &str = concat!(
    include_str!("queries/pull.graphql"),
    include_str!("queries/thread-fields.graphql"),
    include_str!("queries/check-fields.graphql")
);
const THREADS: &str = concat!(
    include_str!("queries/pull-threads.graphql"),
    include_str!("queries/thread-fields.graphql")
);
const CHECKS: &str = concat!(
    include_str!("queries/pull-checks.graphql"),
    include_str!("queries/check-fields.graphql")
);
const ISSUE: &str = include_str!("queries/issue.graphql");
const PAGES: usize = 10;

#[derive(Deserialize)]
struct Reply<T> {
    repository: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
    #[serde(default = "Vec::new")]
    nodes: Vec<Option<T>>,
    page_info: Option<PageInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

impl<T> Page<T> {
    fn items(self) -> impl Iterator<Item = T> {
        self.nodes.into_iter().flatten()
    }

    fn after(&self) -> Option<String> {
        self.page_info
            .as_ref()
            .filter(|info| info.has_next_page)?
            .end_cursor
            .clone()
    }

    fn extend(&mut self, page: Self) {
        self.nodes.extend(page.nodes);
        self.page_info = page.page_info;
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadsRepo {
    pull_request: Option<ThreadsPull>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadsPull {
    review_threads: Option<Page<RawThread>>,
}

#[derive(Deserialize)]
struct ChecksRepo {
    object: Option<Commit>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRepo {
    #[serde(flatten)]
    methods: Methods,
    #[serde(default)]
    auto_merge_allowed: bool,
    pull_request: Option<RawItem>,
}

#[derive(Deserialize)]
struct Methods {
    #[serde(rename = "mergeCommitAllowed")]
    merge: bool,
    #[serde(rename = "squashMergeAllowed")]
    squash: bool,
    #[serde(rename = "rebaseMergeAllowed")]
    rebase: bool,
}

#[derive(Deserialize)]
struct IssueRepo {
    issue: Option<RawItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    id: String,
    number: u32,
    title: String,
    url: String,
    state: String,
    created_at: String,
    body: String,
    #[serde(rename = "bodyHTML")]
    html: String,
    viewer_can_update: bool,
    author: Option<Login>,
    labels: Option<Nodes<RawLabel>>,
    assignees: Option<Nodes<Login>>,
    comments: Option<Nodes<RawComment>>,
    reviews: Option<Nodes<RawComment>>,
    #[serde(alias = "closedByPullRequestsReferences")]
    closing_issues_references: Option<Nodes<RawLinked>>,
    is_draft: Option<bool>,
    head_ref_name: Option<String>,
    head_repository_owner: Option<Login>,
    head_ref_oid: Option<String>,
    base_ref_name: Option<String>,
    is_cross_repository: Option<bool>,
    review_decision: Option<String>,
    merge_state_status: Option<String>,
    auto_merge_request: Option<AutoMerge>,
    review_threads: Option<Page<RawThread>>,
    additions: Option<u32>,
    deletions: Option<u32>,
    changed_files: Option<u32>,
    review_requests: Option<Nodes<Requested>>,
    files: Option<Nodes<RawFile>>,
    commits: Option<Page<CommitNode>>,
}

#[derive(Deserialize)]
struct RawLinked {
    number: u32,
    title: String,
    url: String,
    state: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Requested {
    requested_reviewer: Option<Reviewer>,
}

#[derive(Deserialize)]
struct Reviewer {
    login: Option<String>,
}

#[derive(Deserialize)]
struct RawFile {
    path: String,
    additions: u32,
    deletions: u32,
}

impl Gh {
    pub async fn pull(&self, repo: &Repo, number: u32) -> Result<Detail, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name, "number": number });
        let mut reply: Reply<PullRepo> = self.graphql(&host, PULL, variables.clone()).await?;
        if let Some(raw) = reply
            .repository
            .as_mut()
            .and_then(|repo| repo.pull_request.as_mut())
        {
            self.remaining(&host, &variables, raw).await?;
        }

        pull(reply, number)
    }

    async fn remaining(
        &self,
        host: &str,
        variables: &Value,
        raw: &mut RawItem,
    ) -> Result<(), Error> {
        for _ in 0..PAGES {
            let Some(after) = raw.review_threads.as_ref().and_then(Page::after) else {
                break;
            };
            let mut variables = variables.clone();
            variables["after"] = json!(after);
            let reply: Reply<ThreadsRepo> = self.graphql(host, THREADS, variables).await?;
            add_threads(raw, reply);
        }

        for _ in 0..PAGES {
            let Some(after) = contexts(raw).and_then(|page| page.after()) else {
                break;
            };
            let mut variables = variables.clone();
            variables["after"] = json!(after);
            variables["oid"] = json!(raw.head_ref_oid);
            let reply: Reply<ChecksRepo> = self.graphql(host, CHECKS, variables).await?;
            add_checks(raw, reply);
        }

        Ok(())
    }

    pub async fn issue(&self, repo: &Repo, number: u32) -> Result<Detail, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name, "number": number });
        let reply: Reply<IssueRepo> = self.graphql(&host, ISSUE, variables).await?;
        let raw = reply
            .repository
            .and_then(|repo| repo.issue)
            .ok_or_else(|| missing(number))?;

        Ok(detail(raw, Rules::default()))
    }
}

fn pull(reply: Reply<PullRepo>, number: u32) -> Result<Detail, Error> {
    let repo = reply.repository.ok_or_else(|| missing(number))?;
    let methods = [
        (repo.methods.squash, MergeMethod::Squash),
        (repo.methods.merge, MergeMethod::Merge),
        (repo.methods.rebase, MergeMethod::Rebase),
    ]
    .into_iter()
    .filter_map(|(allowed, method)| allowed.then_some(method))
    .collect();
    let raw = repo.pull_request.ok_or_else(|| missing(number))?;

    Ok(detail(
        raw,
        Rules {
            methods,
            automerge: repo.auto_merge_allowed,
        },
    ))
}

fn missing(number: u32) -> Error {
    Error::Gh(format!("#{number} was not found"))
}

#[derive(Default)]
struct Rules {
    methods: Vec<MergeMethod>,
    automerge: bool,
}

#[cfg(test)]
mod tests;
