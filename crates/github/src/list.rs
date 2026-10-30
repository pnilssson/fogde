use protocol::github::{Filter, ItemState, Label, Listed, PullSummary, ReviewState, Section};
use protocol::repos::Repo;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::Error;
use crate::api::checked;
use crate::gh::Gh;
use crate::list::checks::{Rollup, summary};

const PULLS: &str = concat!(
    include_str!("queries/pulls.graphql"),
    include_str!("queries/pull-fields.graphql")
);
const ISSUES: &str = concat!(
    include_str!("queries/issues.graphql"),
    include_str!("queries/issue-fields.graphql")
);
const OPEN_PULLS: &str = concat!(
    include_str!("queries/open-pulls.graphql"),
    include_str!("queries/pull-fields.graphql")
);
const OPEN_ISSUES: &str = concat!(
    include_str!("queries/open-issues.graphql"),
    include_str!("queries/issue-fields.graphql")
);
const SECTIONS: [(Section, &str); 3] = [
    (Section::Review, "/review/nodes"),
    (Section::Mine, "/mine/nodes"),
    (Section::Assigned, "/assigned/nodes"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Pull,
    Issue,
}

#[derive(Deserialize)]
pub(crate) struct Nodes<T> {
    #[serde(default = "Vec::new")]
    nodes: Vec<Option<T>>,
}

impl<T> Nodes<T> {
    pub(crate) fn items(self) -> impl Iterator<Item = T> {
        self.nodes.into_iter().flatten()
    }
}

#[derive(Deserialize)]
pub(crate) struct Login {
    pub(crate) login: String,
}

#[derive(Deserialize)]
pub(crate) struct RawLabel {
    name: String,
    color: String,
}

#[derive(Deserialize)]
struct Count {
    #[serde(rename = "totalCount")]
    total: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    number: u32,
    title: String,
    url: String,
    state: String,
    updated_at: String,
    author: Option<Login>,
    labels: Option<Nodes<RawLabel>>,
    comments: Count,
    is_draft: Option<bool>,
    head_ref_name: Option<String>,
    head_ref_oid: Option<String>,
    head_repository_owner: Option<Login>,
    base_ref_name: Option<String>,
    review_decision: Option<String>,
    additions: Option<u32>,
    deletions: Option<u32>,
    commits: Option<Nodes<CommitNode>>,
}

#[derive(Deserialize)]
struct CommitNode {
    commit: Commit,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    status_check_rollup: Option<Rollup>,
}

impl Gh {
    pub async fn pulls(&self, repo: &Repo, filter: Filter) -> Result<Vec<Listed>, Error> {
        self.list(repo, filter, Kind::Pull).await
    }

    pub async fn issues(&self, repo: &Repo, filter: Filter) -> Result<Vec<Listed>, Error> {
        self.list(repo, filter, Kind::Issue).await
    }

    async fn list(&self, repo: &Repo, filter: Filter, kind: Kind) -> Result<Vec<Listed>, Error> {
        let host = checked(repo)?;
        let (query, variables) = match (filter, kind) {
            (Filter::Open, Kind::Pull) => (OPEN_PULLS, searches(repo, kind)),
            (Filter::Open, Kind::Issue) => (OPEN_ISSUES, searches(repo, kind)),
            (Filter::Closed, Kind::Pull) => (PULLS, json!({ "states": ["CLOSED", "MERGED"] })),
            (Filter::Closed, Kind::Issue) => (ISSUES, json!({ "states": ["CLOSED"] })),
        };
        let mut variables = variables;
        variables["owner"] = json!(repo.owner);
        variables["name"] = json!(repo.name);

        let reply: Value = self.graphql(&host, query, variables).await?;

        parse(&reply)
    }
}

fn searches(repo: &Repo, kind: Kind) -> Value {
    let search = |person: &str| {
        let kind = if kind == Kind::Pull {
            "is:pr"
        } else {
            "is:issue"
        };
        format!(
            "repo:{}/{} {kind} is:open sort:updated-desc {person}",
            repo.owner, repo.name
        )
    };
    let mut variables = json!({
        "mine": search("author:@me"),
        "assigned": search("assignee:@me"),
    });
    if kind == Kind::Pull {
        variables["review"] = json!(search("review-requested:@me"));
    }

    variables
}

fn parse(reply: &Value) -> Result<Vec<Listed>, Error> {
    let mut listed: Vec<Listed> = Vec::new();
    let sections = SECTIONS.into_iter().chain([
        (Section::Other, "/repository/pullRequests/nodes"),
        (Section::Other, "/repository/issues/nodes"),
    ]);

    for (section, path) in sections {
        let nodes = reply.pointer(path).and_then(Value::as_array);
        for node in nodes.into_iter().flatten() {
            if node.get("number").is_none() {
                continue;
            }
            let item = item(node, section)?;
            if !listed.iter().any(|known| known.number == item.number) {
                listed.push(item);
            }
        }
    }

    Ok(listed)
}

fn item(node: &Value, section: Section) -> Result<Listed, serde_json::Error> {
    let raw = RawItem::deserialize(node)?;
    let pull = raw.head_ref_name.map(|head| PullSummary {
        draft: raw.is_draft.unwrap_or(false),
        head,
        sha: raw.head_ref_oid.unwrap_or_default(),
        owner: raw.head_repository_owner.map(|owner| owner.login),
        base: raw.base_ref_name.unwrap_or_default(),
        review: raw.review_decision.as_deref().and_then(review),
        checks: raw
            .commits
            .and_then(|commits| commits.items().next())
            .and_then(|node| node.commit.status_check_rollup)
            .and_then(summary),
        added: raw.additions.unwrap_or(0),
        removed: raw.deletions.unwrap_or(0),
    });

    Ok(Listed {
        number: raw.number,
        title: raw.title,
        url: raw.url,
        state: state(&raw.state),
        author: raw.author.map(|author| author.login),
        labels: labels(raw.labels),
        comments: raw.comments.total,
        updated: raw.updated_at,
        section,
        pull,
    })
}

pub(crate) fn labels(labels: Option<Nodes<RawLabel>>) -> Vec<Label> {
    labels
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .map(|label| Label {
            name: label.name,
            color: label.color,
        })
        .collect()
}

pub(crate) fn logins(people: Option<Nodes<Login>>) -> Vec<String> {
    people
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .map(|person| person.login)
        .collect()
}

pub(crate) fn state(state: &str) -> ItemState {
    match state {
        "MERGED" => ItemState::Merged,
        "CLOSED" => ItemState::Closed,
        _ => ItemState::Open,
    }
}

pub(crate) fn review(decision: &str) -> Option<ReviewState> {
    match decision {
        "APPROVED" => Some(ReviewState::Approved),
        "CHANGES_REQUESTED" => Some(ReviewState::Changes),
        "REVIEW_REQUIRED" => Some(ReviewState::Required),
        _ => None,
    }
}

mod checks;
#[cfg(test)]
mod tests;
