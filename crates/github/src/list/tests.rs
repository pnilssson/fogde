use super::*;
use protocol::github::{ChecksState, ChecksSummary};

const OPEN_PULLS: &str = r#"{
  "repository":{"pullRequests":{"nodes":[
    {"number":42,"title":"Add login","url":"https://github.com/acme/app/pull/42","state":"OPEN","updatedAt":"2026-10-28T12:50:03Z","isDraft":true,"headRefName":"feature/login","headRefOid":"3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345","headRepositoryOwner":{"login":"acme"},"baseRefName":"main","reviewDecision":"REVIEW_REQUIRED","additions":509,"deletions":69,"author":{"login":"dev"},"labels":{"nodes":[{"name":"enhancement","color":"0dd8ac"}]},"comments":{"totalCount":3},"commits":{"nodes":[{"commit":{"statusCheckRollup":{"state":"FAILURE","contexts":{"totalCount":25,"checkRunCountsByState":[{"state":"SUCCESS","count":10},{"state":"SKIPPED","count":13},{"state":"CANCELLED","count":1},{"state":"IN_PROGRESS","count":0}],"statusContextCountsByState":[{"state":"FAILURE","count":1},{"state":"PENDING","count":0}]}}}}]}},
    {"number":41,"title":"Fix typo","url":"https://github.com/acme/app/pull/41","state":"OPEN","updatedAt":"2026-10-27T09:14:33Z","isDraft":false,"headRefName":"typo","headRepositoryOwner":null,"baseRefName":"main","reviewDecision":null,"additions":1,"deletions":1,"author":null,"labels":{"nodes":[]},"comments":{"totalCount":0},"commits":{"nodes":[{"commit":{"statusCheckRollup":null}}]}},
    {"number":40,"title":"Docs","url":"https://github.com/acme/app/pull/40","state":"OPEN","updatedAt":"2026-10-26T09:14:33Z","isDraft":false,"headRefName":"docs","headRepositoryOwner":{"login":"dev"},"baseRefName":"main","reviewDecision":"APPROVED","additions":3,"deletions":0,"author":{"login":"dev"},"labels":{"nodes":[]},"comments":{"totalCount":0},"commits":{"nodes":[]}},
    null
  ]}},
  "review":{"nodes":[]},
  "mine":{"nodes":[
    {"number":40,"title":"Docs","url":"https://github.com/acme/app/pull/40","state":"OPEN","updatedAt":"2026-10-26T09:14:33Z","isDraft":false,"headRefName":"docs","headRepositoryOwner":{"login":"dev"},"baseRefName":"main","reviewDecision":"APPROVED","additions":3,"deletions":0,"author":{"login":"dev"},"labels":{"nodes":[]},"comments":{"totalCount":0},"commits":{"nodes":[]}},
    {}
  ]},
  "assigned":{"nodes":[
    {"number":42,"title":"Add login","url":"https://github.com/acme/app/pull/42","state":"OPEN","updatedAt":"2026-10-28T12:50:03Z","isDraft":true,"headRefName":"feature/login","headRefOid":"3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345","headRepositoryOwner":{"login":"acme"},"baseRefName":"main","reviewDecision":"REVIEW_REQUIRED","additions":509,"deletions":69,"author":{"login":"dev"},"labels":{"nodes":[]},"comments":{"totalCount":3},"commits":{"nodes":[]}}
  ]}
}"#;

const CLOSED_ISSUES: &str = r#"{"repository":{"issues":{"nodes":[
  {"number":7,"title":"Login fails on Windows","url":"https://github.com/acme/app/issues/7","state":"CLOSED","updatedAt":"2026-10-26T08:00:00Z","author":{"login":"dev"},"labels":{"nodes":[{"name":"bug","color":"d73a4a"}]},"comments":{"totalCount":1}}
]}}}"#;

#[test]
fn groups_open_pull_requests_by_what_needs_you_once_each() {
    let listed = parse(&serde_json::from_str(OPEN_PULLS).unwrap()).unwrap();

    let order: Vec<(u32, Section)> = listed
        .iter()
        .map(|item| (item.number, item.section))
        .collect();
    assert_eq!(
        order,
        [
            (40, Section::Mine),
            (42, Section::Assigned),
            (41, Section::Other)
        ]
    );
}

#[test]
fn reads_pull_requests_with_reviews_check_counts_and_missing_authors() {
    let listed = parse(&serde_json::from_str(OPEN_PULLS).unwrap()).unwrap();

    let login = listed.iter().find(|item| item.number == 42).unwrap();
    assert_eq!(login.author.as_deref(), Some("dev"));
    assert_eq!(
        login.pull.as_ref().unwrap(),
        &PullSummary {
            draft: true,
            head: "feature/login".to_owned(),
            sha: "3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345".to_owned(),
            owner: Some("acme".to_owned()),
            base: "main".to_owned(),
            review: Some(ReviewState::Required),
            checks: None,
            added: 509,
            removed: 69,
        }
    );

    let typo = listed.iter().find(|item| item.number == 41).unwrap();
    assert_eq!(typo.author, None);
    let pull = typo.pull.as_ref().unwrap();
    assert_eq!((pull.review, pull.checks), (None, None));
}

#[test]
fn counts_checks_without_skipped_ones_and_treats_cancelled_as_failed() {
    let reply: Value = serde_json::from_str(OPEN_PULLS).unwrap();
    let node = reply
        .pointer("/repository/pullRequests/nodes/0")
        .unwrap()
        .clone();

    let listed = item(&node, Section::Other).unwrap();

    assert_eq!(
        listed.pull.unwrap().checks,
        Some(ChecksSummary {
            state: ChecksState::Failure,
            passed: 10,
            failed: 2,
            total: 12,
        })
    );
}

#[test]
fn reads_closed_issues() {
    let listed = parse(&serde_json::from_str(CLOSED_ISSUES).unwrap()).unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].state, ItemState::Closed);
    assert_eq!(listed[0].section, Section::Other);
    assert_eq!(listed[0].pull, None);
    assert_eq!(listed[0].labels[0].name, "bug");
}

#[test]
fn reads_an_empty_or_missing_repository() {
    assert_eq!(parse(&json!({ "repository": null })).unwrap(), []);
    assert_eq!(
        parse(&json!({ "repository": null, "mine": { "nodes": [] } })).unwrap(),
        []
    );
}

#[test]
fn searches_only_the_repo_and_the_person_asked_for() {
    let repo = Repo {
        host: "github.com".to_owned(),
        owner: "acme".to_owned(),
        name: "app".to_owned(),
    };

    assert_eq!(
        searches(&repo, Kind::Pull),
        json!({
            "review": "repo:acme/app is:pr is:open sort:updated-desc review-requested:@me",
            "mine": "repo:acme/app is:pr is:open sort:updated-desc author:@me",
            "assigned": "repo:acme/app is:pr is:open sort:updated-desc assignee:@me",
        })
    );
    assert_eq!(
        searches(&repo, Kind::Issue),
        json!({
            "mine": "repo:acme/app is:issue is:open sort:updated-desc author:@me",
            "assigned": "repo:acme/app is:issue is:open sort:updated-desc assignee:@me",
        })
    );
}
