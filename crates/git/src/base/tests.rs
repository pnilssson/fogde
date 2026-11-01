use std::net::TcpListener;
use std::time::Instant;

use super::*;
use crate::testing::repo;

fn remotes(entries: &[(&str, Option<&str>)]) -> Vec<Remote> {
    entries
        .iter()
        .map(|(name, resolved)| Remote {
            name: (*name).to_owned(),
            url: format!("/repos/{name}"),
            resolved: resolved.map(str::to_owned),
        })
        .collect()
}

fn chosen(entries: &[(&str, Option<&str>)]) -> Option<String> {
    remote(&remotes(entries)).map(|remote| remote.name.clone())
}

#[test]
fn ranks_upstream_then_github_then_origin_then_the_only_remote() {
    assert_eq!(
        chosen(&[("origin", None), ("upstream", None)]).as_deref(),
        Some("upstream")
    );
    assert_eq!(
        chosen(&[("origin", None), ("github", None), ("fork", None)]).as_deref(),
        Some("github")
    );
    assert_eq!(
        chosen(&[("fork", None), ("origin", None)]).as_deref(),
        Some("origin")
    );
    assert_eq!(chosen(&[("fork", None)]).as_deref(), Some("fork"));
    assert_eq!(chosen(&[("fork", None), ("mirror", None)]), None);
    assert_eq!(chosen(&[]), None);
}

#[test]
fn prefers_the_remote_gh_resolved() {
    assert_eq!(
        chosen(&[("upstream", None), ("origin", Some("base"))]).as_deref(),
        Some("origin")
    );
}

#[tokio::test]
async fn gives_up_on_a_fetch_that_hangs() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    let silent = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("git://{}/app", silent.local_addr().unwrap());
    git(dir.path(), ["remote", "add", "origin", &url])
        .await
        .unwrap();
    let started = Instant::now();

    let error = fetch_within(dir.path(), Duration::from_secs(2))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("too long"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(30));
}

#[tokio::test]
async fn falls_back_to_the_next_remote_without_a_mainline() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path()).await;
    git(
        dir.path(),
        ["commit", "--quiet", "--allow-empty", "-m", "first"],
    )
    .await
    .unwrap();
    for name in ["upstream", "origin"] {
        let url = format!("/repos/{name}");
        git(dir.path(), ["remote", "add", name, &url])
            .await
            .unwrap();
    }
    git(
        dir.path(),
        ["update-ref", "refs/remotes/upstream/develop", "HEAD"],
    )
    .await
    .unwrap();
    git(
        dir.path(),
        ["update-ref", "refs/remotes/origin/main", "HEAD"],
    )
    .await
    .unwrap();

    let found = mainline(dir.path()).await.unwrap();

    assert_eq!(found.reference, "refs/remotes/origin/main");
}
