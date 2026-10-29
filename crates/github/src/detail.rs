use protocol::github::{
    Comment, Detail, FileStat, Linked, MergeMethod, PullDetail, RepoOptions, Verdict,
};
use protocol::repos::Repo;
use serde::Deserialize;
use serde_json::json;

use crate::Error;
use crate::api::checked;
use crate::gh::Gh;
use crate::list::{Login, Nodes, RawLabel, labels, logins, review, state};

mod checks;

use checks::{CommitNode, check, merge};

const PULL: &str = include_str!("queries/pull.graphql");
const ISSUE: &str = include_str!("queries/issue.graphql");
const OPTIONS: &str = include_str!("queries/options.graphql");

#[derive(Deserialize)]
struct Reply<T> {
    repository: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRepo {
    merge_commit_allowed: bool,
    squash_merge_allowed: bool,
    rebase_merge_allowed: bool,
    pull_request: Option<RawItem>,
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
    additions: Option<u32>,
    deletions: Option<u32>,
    changed_files: Option<u32>,
    review_requests: Option<Nodes<Requested>>,
    files: Option<Nodes<RawFile>>,
    commits: Option<Nodes<CommitNode>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawComment {
    id: String,
    url: String,
    created_at: Option<String>,
    submitted_at: Option<String>,
    state: Option<String>,
    #[serde(rename = "bodyHTML")]
    html: String,
    author: Option<Login>,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionsRepo {
    default_branch_ref: Option<Branch>,
    labels: Option<Nodes<RawLabel>>,
    assignable_users: Option<Nodes<Login>>,
}

#[derive(Deserialize)]
struct Branch {
    name: String,
}

impl Gh {
    pub async fn pull(&self, repo: &Repo, number: u32) -> Result<Detail, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name, "number": number });
        let reply: Reply<PullRepo> = self.graphql(&host, PULL, variables).await?;

        pull(reply, number)
    }

    pub async fn issue(&self, repo: &Repo, number: u32) -> Result<Detail, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name, "number": number });
        let reply: Reply<IssueRepo> = self.graphql(&host, ISSUE, variables).await?;
        let raw = reply
            .repository
            .and_then(|repo| repo.issue)
            .ok_or_else(|| missing(number))?;

        Ok(detail(raw, Vec::new()))
    }

    pub async fn options(&self, repo: &Repo) -> Result<RepoOptions, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name });
        let reply: Reply<OptionsRepo> = self.graphql(&host, OPTIONS, variables).await?;
        let raw = reply
            .repository
            .ok_or_else(|| Error::Gh(format!("{}/{} was not found", repo.owner, repo.name)))?;

        Ok(RepoOptions {
            labels: labels(raw.labels),
            people: logins(raw.assignable_users),
            branch: raw.default_branch_ref.map(|branch| branch.name),
        })
    }
}

fn pull(reply: Reply<PullRepo>, number: u32) -> Result<Detail, Error> {
    let repo = reply.repository.ok_or_else(|| missing(number))?;
    let methods = [
        (repo.squash_merge_allowed, MergeMethod::Squash),
        (repo.merge_commit_allowed, MergeMethod::Merge),
        (repo.rebase_merge_allowed, MergeMethod::Rebase),
    ]
    .into_iter()
    .filter_map(|(allowed, method)| allowed.then_some(method))
    .collect();
    let raw = repo.pull_request.ok_or_else(|| missing(number))?;

    Ok(detail(raw, methods))
}

fn missing(number: u32) -> Error {
    Error::Gh(format!("#{number} was not found"))
}

fn detail(mut raw: RawItem, methods: Vec<MergeMethod>) -> Detail {
    let reviews = raw.reviews.take().map(Nodes::items).into_iter().flatten();
    let mut comments: Vec<Comment> = raw
        .comments
        .take()
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .chain(reviews)
        .filter_map(comment)
        .collect();
    comments.sort_by(|a, b| a.created.cmp(&b.created));

    let linked = raw
        .closing_issues_references
        .take()
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .map(|link| Linked {
            number: link.number,
            title: link.title,
            url: link.url,
            state: state(&link.state),
        })
        .collect();

    let pull = pull_detail(&mut raw, methods);

    Detail {
        id: raw.id,
        number: raw.number,
        title: raw.title,
        url: raw.url,
        state: state(&raw.state),
        author: raw.author.map(|author| author.login),
        created: raw.created_at,
        body: raw.body,
        html: raw.html,
        labels: labels(raw.labels),
        assignees: logins(raw.assignees),
        editable: raw.viewer_can_update,
        comments,
        linked,
        pull,
    }
}

fn pull_detail(raw: &mut RawItem, methods: Vec<MergeMethod>) -> Option<PullDetail> {
    let head = raw.head_ref_name.take()?;
    let checks = raw
        .commits
        .take()
        .and_then(|commits| commits.items().next())
        .and_then(|node| node.commit.status_check_rollup)
        .map(|rollup| rollup.contexts.items().filter_map(check).collect())
        .unwrap_or_default();

    Some(PullDetail {
        draft: raw.is_draft.unwrap_or(false),
        head,
        owner: raw.head_repository_owner.take().map(|owner| owner.login),
        sha: raw.head_ref_oid.take().unwrap_or_default(),
        base: raw.base_ref_name.take().unwrap_or_default(),
        cross: raw.is_cross_repository.unwrap_or(false),
        review: raw.review_decision.as_deref().and_then(review),
        merge: merge(raw.merge_state_status.as_deref().unwrap_or_default()),
        methods,
        reviewers: raw
            .review_requests
            .take()
            .map(Nodes::items)
            .into_iter()
            .flatten()
            .filter_map(|request| request.requested_reviewer?.login)
            .collect(),
        checks,
        files: raw
            .files
            .take()
            .map(Nodes::items)
            .into_iter()
            .flatten()
            .map(|file| FileStat {
                path: file.path,
                added: file.additions,
                removed: file.deletions,
            })
            .collect(),
        added: raw.additions.unwrap_or(0),
        removed: raw.deletions.unwrap_or(0),
        changed: raw.changed_files.unwrap_or(0),
    })
}

fn comment(raw: RawComment) -> Option<Comment> {
    let verdict = match raw.state.as_deref() {
        None => None,
        Some("APPROVED") => Some(Verdict::Approved),
        Some("CHANGES_REQUESTED") => Some(Verdict::Changes),
        Some("DISMISSED") => Some(Verdict::Dismissed),
        Some("COMMENTED") if !raw.html.trim().is_empty() => Some(Verdict::Commented),
        Some(_) => return None,
    };

    Some(Comment {
        id: raw.id,
        author: raw.author.map(|author| author.login),
        html: raw.html,
        created: raw.created_at.or(raw.submitted_at)?,
        url: raw.url,
        verdict,
    })
}

#[cfg(test)]
mod tests;
