use protocol::github::{Check, CheckState, ItemState, MergeState, ReviewState};

use super::*;

const PULL_REPLY: &str = r#"{"repository":{"mergeCommitAllowed":false,"squashMergeAllowed":true,"rebaseMergeAllowed":true,"pullRequest":{
  "id":"PR_kwDOAAAAAc8AAAABJwiFQg","number":42,"title":"Add login","url":"https://github.com/acme/app/pull/42","state":"OPEN","createdAt":"2026-10-21T10:05:39Z",
  "body":"Adds login.\n\nFixes #7","bodyHTML":"<p dir=\"auto\">Adds login.</p>","viewerCanUpdate":true,"isDraft":false,
  "headRefName":"feature/login","headRepositoryOwner":{"login":"acme"},"headRefOid":"3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345","baseRefName":"main","isCrossRepository":false,"reviewDecision":"CHANGES_REQUESTED","mergeStateStatus":"BEHIND",
  "additions":120,"deletions":8,"changedFiles":2,"author":{"login":"dev"},
  "labels":{"nodes":[{"name":"enhancement","color":"0dd8ac"}]},
  "assignees":{"nodes":[{"login":"dev"}]},
  "reviewRequests":{"nodes":[{"requestedReviewer":{"login":"reviewer"}},{"requestedReviewer":{}}]},
  "comments":{"nodes":[{"id":"IC_2","url":"https://github.com/acme/app/pull/42#issuecomment-2","createdAt":"2026-10-23T08:00:00Z","bodyHTML":"<p>Thanks</p>","author":null}]},
  "reviews":{"nodes":[
    {"id":"PRR_1","url":"https://github.com/acme/app/pull/42#pullrequestreview-1","state":"CHANGES_REQUESTED","submittedAt":"2026-10-22T08:00:00Z","bodyHTML":"<p>Please add tests</p>","author":{"login":"reviewer"}},
    {"id":"PRR_2","url":"https://github.com/acme/app/pull/42#pullrequestreview-2","state":"COMMENTED","submittedAt":"2026-10-22T09:00:00Z","bodyHTML":"","author":{"login":"reviewer"}},
    {"id":"PRR_3","url":"https://github.com/acme/app/pull/42#pullrequestreview-3","state":"PENDING","submittedAt":null,"bodyHTML":"<p>Draft</p>","author":{"login":"dev"}},
    {"id":"PRR_4","url":"https://github.com/acme/app/pull/42#pullrequestreview-4","state":"APPROVED","submittedAt":"2026-10-24T08:00:00Z","bodyHTML":"","author":{"login":"reviewer"}}
  ]},
  "closingIssuesReferences":{"nodes":[{"number":7,"title":"Login fails","url":"https://github.com/acme/app/issues/7","state":"OPEN"}]},
  "files":{"nodes":[{"path":"src/login.rs","additions":110,"deletions":0},{"path":"src/app.rs","additions":10,"deletions":8}]},
  "commits":{"nodes":[{"commit":{"statusCheckRollup":{"contexts":{"nodes":[
    {"__typename":"CheckRun","name":"test","status":"COMPLETED","conclusion":"FAILURE","detailsUrl":"https://github.com/acme/app/actions/runs/39586731133/job/1","checkSuite":{"workflowRun":{"databaseId":39586731133}}},
    {"__typename":"CheckRun","name":"lint","status":"IN_PROGRESS","conclusion":null,"detailsUrl":null,"checkSuite":{"workflowRun":null}},
    {"__typename":"CheckRun","name":"label","status":"COMPLETED","conclusion":"SKIPPED","detailsUrl":null,"checkSuite":null},
    {"__typename":"StatusContext","context":"ci/deploy","state":"PENDING","targetUrl":"https://ci.example.com/1"}
  ]}}}}]}
}}}"#;

const ISSUE_REPLY: &str = r#"{"repository":{"issue":{
  "id":"I_kwDOAAAAAc8AAAAAAAAAAg","number":7,"title":"Login fails","url":"https://github.com/acme/app/issues/7","state":"CLOSED","createdAt":"2026-10-20T10:00:00Z",
  "body":"It fails.","bodyHTML":"<p dir=\"auto\">It fails.</p>","viewerCanUpdate":false,"author":{"login":"dev"},
  "labels":{"nodes":[]},"assignees":{"nodes":[]},
  "comments":{"nodes":[{"id":"IC_1","url":"https://github.com/acme/app/issues/7#issuecomment-1","createdAt":"2026-10-20T11:00:00Z","bodyHTML":"<p>Same here</p>","author":{"login":"reviewer"}}]},
  "closedByPullRequestsReferences":{"nodes":[{"number":42,"title":"Add login","url":"https://github.com/acme/app/pull/42","state":"MERGED"}]}
}}}"#;

#[test]
fn reads_a_pull_request_with_its_conversation_and_checks() {
    let detail = pull(serde_json::from_str(PULL_REPLY).unwrap(), 42).unwrap();

    assert_eq!(detail.number, 42);
    assert!(detail.editable);
    assert_eq!(detail.assignees, ["dev"]);
    assert_eq!(
        detail
            .comments
            .iter()
            .map(|comment| (comment.id.as_str(), comment.verdict))
            .collect::<Vec<_>>(),
        [
            ("PRR_1", Some(Verdict::Changes)),
            ("IC_2", None),
            ("PRR_4", Some(Verdict::Approved)),
        ]
    );
    assert_eq!(detail.comments[1].author, None);
    assert_eq!(detail.linked[0].number, 7);

    let pull = detail.pull.unwrap();
    assert_eq!(pull.head, "feature/login");
    assert_eq!(pull.owner.as_deref(), Some("acme"));
    assert_eq!(pull.review, Some(ReviewState::Changes));
    assert_eq!(pull.merge, MergeState::Behind);
    assert_eq!(pull.methods, [MergeMethod::Squash, MergeMethod::Rebase]);
    assert_eq!(pull.reviewers, ["reviewer"]);
    assert_eq!((pull.added, pull.removed, pull.changed), (120, 8, 2));
    assert_eq!(pull.files.len(), 2);
    assert_eq!(
        pull.checks,
        [
            Check {
                name: "test".to_owned(),
                state: CheckState::Failure,
                url: Some("https://github.com/acme/app/actions/runs/39586731133/job/1".to_owned()),
                run: Some(39_586_731_133),
            },
            Check {
                name: "lint".to_owned(),
                state: CheckState::Pending,
                url: None,
                run: None,
            },
            Check {
                name: "label".to_owned(),
                state: CheckState::Skipped,
                url: None,
                run: None,
            },
            Check {
                name: "ci/deploy".to_owned(),
                state: CheckState::Pending,
                url: Some("https://ci.example.com/1".to_owned()),
                run: None,
            },
        ]
    );
}

#[test]
fn reads_an_issue_with_the_pull_requests_that_close_it() {
    let reply: Reply<IssueRepo> = serde_json::from_str(ISSUE_REPLY).unwrap();
    let detail = detail(reply.repository.unwrap().issue.unwrap(), Vec::new());

    assert_eq!(detail.state, ItemState::Closed);
    assert!(!detail.editable);
    assert_eq!(detail.pull, None);
    assert_eq!(detail.comments.len(), 1);
    assert_eq!(detail.linked[0].state, ItemState::Merged);
}

#[test]
fn reports_a_missing_pull_request() {
    let reply: Reply<PullRepo> = serde_json::from_str(
        r#"{"repository":{"mergeCommitAllowed":true,"squashMergeAllowed":true,"rebaseMergeAllowed":true,"pullRequest":null}}"#,
    )
    .unwrap();

    assert!(pull(reply, 9).is_err());
}
