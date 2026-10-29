use super::*;

fn body(request: &Request) -> String {
    request
        .body
        .as_ref()
        .map(Value::to_string)
        .unwrap_or_default()
}

#[test]
fn builds_comments_reviews_and_merges() {
    let comment = comment(42, "  Looks good  ").unwrap();
    assert_eq!(
        (comment.verb, comment.path.as_str()),
        (Verb::Post, "issues/42/comments")
    );
    assert_eq!(body(&comment), r#"{"body":"Looks good"}"#);
    assert!(super::comment(42, " ").is_err());

    let approve = review(42, ReviewEvent::Approve, "").unwrap();
    assert_eq!(approve.path, "pulls/42/reviews");
    assert_eq!(body(&approve), r#"{"event":"APPROVE"}"#);
    let changes = review(42, ReviewEvent::Changes, "Add tests").unwrap();
    assert_eq!(
        body(&changes),
        r#"{"body":"Add tests","event":"REQUEST_CHANGES"}"#
    );
    assert!(review(42, ReviewEvent::Changes, "").is_err());
    assert!(review(42, ReviewEvent::Comment, "").is_err());
    assert!(merge(42, MergeMethod::Squash, "").is_err());
    assert!(merge(42, MergeMethod::Squash, "main").is_err());

    let squash = merge(
        42,
        MergeMethod::Squash,
        "3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345",
    )
    .unwrap();
    assert_eq!(
        (squash.verb, squash.path.as_str()),
        (Verb::Put, "pulls/42/merge")
    );
    assert_eq!(
        body(&squash),
        r#"{"merge_method":"squash","sha":"3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345"}"#
    );
}

#[test]
fn deletes_branches_with_encoded_names() {
    let request = delete_branch("feature/login").unwrap();
    assert_eq!(
        (request.verb, request.path.as_str()),
        (Verb::Delete, "git/refs/heads/feature/login")
    );
    assert_eq!(request.body, None);

    for (name, path) in [
        ("fix#7", "git/refs/heads/fix%237"),
        ("release@2", "git/refs/heads/release%402"),
        ("feature/a%b", "git/refs/heads/feature/a%25b"),
        ("feature/über", "git/refs/heads/feature/%C3%BCber"),
        ("dev+login", "git/refs/heads/dev%2Blogin"),
    ] {
        assert_eq!(delete_branch(name).unwrap().path, path, "{name}");
    }
}

#[test]
fn refuses_names_git_rejects() {
    for name in [
        "",
        "-x",
        "@",
        "../main",
        "a//b",
        "/a",
        "a/",
        "a.",
        ".a",
        "a/.b",
        "a b",
        "a\tb",
        "a\u{7f}b",
        "main?x=1",
        "a~1",
        "a^",
        "a:b",
        "a*",
        "a[b",
        r"a\b",
        "a@{1}",
        "topic.lock",
        "a.lock/b",
    ] {
        assert!(delete_branch(name).is_err(), "{name:?}");
    }
}

#[test]
fn builds_state_edit_and_label_changes() {
    let close = set_state(7, false, false);
    assert_eq!((close.verb, close.path.as_str()), (Verb::Patch, "issues/7"));
    assert_eq!(body(&close), r#"{"state":"closed"}"#);
    assert_eq!(set_state(42, true, true).path, "pulls/42");

    let edited = edit(42, true, " Add login ", "Body").unwrap();
    assert_eq!(body(&edited), r#"{"body":"Body","title":"Add login"}"#);
    assert!(edit(42, true, "", "Body").is_err());

    let set = labels(7, &["bug".to_owned(), "needs review".to_owned()]);
    assert_eq!(
        (set.verb, set.path.as_str()),
        (Verb::Put, "issues/7/labels")
    );
    assert_eq!(body(&set), r#"{"labels":["bug","needs review"]}"#);
}

#[test]
fn adds_and_removes_people_in_separate_requests() {
    let requests = reviewers(42, &["dev".to_owned()], &["dev_acme".to_owned()]).unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        (requests[0].verb, requests[0].path.as_str()),
        (Verb::Post, "pulls/42/requested_reviewers")
    );
    assert_eq!(body(&requests[0]), r#"{"reviewers":["dev"]}"#);
    assert_eq!(requests[1].verb, Verb::Delete);
    assert_eq!(body(&requests[1]), r#"{"reviewers":["dev_acme"]}"#);

    assert_eq!(assignees(7, &[], &[]).unwrap(), Vec::<Request>::new());
    assert!(assignees(7, &["../x".to_owned()], &[]).is_err());
}

#[test]
fn builds_new_issues_pulls_and_reruns() {
    let issue = new_issue("Login fails", "").unwrap();
    assert_eq!((issue.verb, issue.path.as_str()), (Verb::Post, "issues"));
    assert!(new_issue(" ", "").is_err());

    let pull = new_pull("Add login", "Fixes #7", "feature/login", None, "main", true).unwrap();
    assert_eq!(pull.path, "pulls");
    assert_eq!(
        body(&pull),
        r#"{"base":"main","body":"Fixes #7","draft":true,"head":"feature/login","title":"Add login"}"#
    );
    assert!(new_pull("Add login", "", "", None, "main", false).is_err());
    assert!(new_pull("Add login", "", "feature/login", None, " ", false).is_err());
    let named = new_pull("Add login", "", "fix#7", None, "release@2", false).unwrap();
    assert!(
        body(&named).contains(r#""base":"release@2","body":"","draft":false,"head":"fix#7""#),
        "{}",
        body(&named)
    );
    assert!(
        new_pull(
            "Add login",
            "",
            "feature/login",
            Some("../x"),
            "main",
            false
        )
        .is_err()
    );

    let fork = new_pull("Add login", "", "feature/login", Some("dev"), "main", false).unwrap();
    assert!(
        body(&fork).contains(r#""head":"dev:feature/login""#),
        "{}",
        body(&fork)
    );

    let rerun = rerun(39_586_731_133);
    assert_eq!(rerun.path, "actions/runs/39586731133/rerun-failed-jobs");
    assert_eq!(rerun.body, None);
}
