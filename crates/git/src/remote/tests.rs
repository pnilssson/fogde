use super::*;

async fn set(root: &Path, entries: &[(&str, &str)]) {
    for (key, value) in entries {
        git(root, ["config", key, value]).await.unwrap();
    }
}

async fn pushes(root: &Path, branch: Option<&str>) -> Option<String> {
    config(root, branch).await.push
}

#[test]
fn reads_remotes_with_dots_in_their_names() {
    let output = "remote.origin.url\ngit@github.com:acme/app.git\0remote.my.fork.url\nhttps://github.com/dev/app\0";

    assert_eq!(
        parse(output, None).remotes,
        [
            Remote {
                name: "origin".to_owned(),
                url: "git@github.com:acme/app.git".to_owned(),
                resolved: None,
            },
            Remote {
                name: "my.fork".to_owned(),
                url: "https://github.com/dev/app".to_owned(),
                resolved: None,
            },
        ]
    );
}

#[tokio::test]
async fn reads_the_default_repo_gh_set() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), ["init", "--quiet"]).await.unwrap();
    git(
        dir.path(),
        ["remote", "add", "origin", "git@github.com:dev/app.git"],
    )
    .await
    .unwrap();
    git(
        dir.path(),
        ["remote", "add", "upstream", "https://github.com/acme/app"],
    )
    .await
    .unwrap();
    git(
        dir.path(),
        ["config", "--add", "remote.origin.gh-resolved", "acme/app"],
    )
    .await
    .unwrap();

    assert_eq!(
        remotes(dir.path()).await,
        [
            Remote {
                name: "origin".to_owned(),
                url: "git@github.com:dev/app.git".to_owned(),
                resolved: Some("acme/app".to_owned()),
            },
            Remote {
                name: "upstream".to_owned(),
                url: "https://github.com/acme/app".to_owned(),
                resolved: None,
            },
        ]
    );
}

#[tokio::test]
async fn reads_the_url_git_fetches_from() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), ["init", "--quiet"]).await.unwrap();
    git(
        dir.path(),
        ["config", "url.git@github.com:.insteadOf", "gh:"],
    )
    .await
    .unwrap();
    git(
        dir.path(),
        ["config", "url.https://github.com/.insteadOf", "g"],
    )
    .await
    .unwrap();
    git(dir.path(), ["remote", "add", "origin", "gh:acme/app"])
        .await
        .unwrap();
    git(
        dir.path(),
        ["config", "--add", "remote.origin.url", "gh:dev/app"],
    )
    .await
    .unwrap();
    let fetched = git(dir.path(), ["remote", "get-url", "origin"])
        .await
        .unwrap();

    let found = remotes(dir.path()).await;

    assert_eq!(
        found,
        [Remote {
            name: "origin".to_owned(),
            url: "git@github.com:acme/app".to_owned(),
            resolved: None,
        }]
    );
    assert_eq!(found[0].url, String::from_utf8_lossy(&fetched).trim());
}

#[tokio::test]
async fn reads_a_branch_remote() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), ["init", "--quiet"]).await.unwrap();
    git(
        dir.path(),
        ["remote", "add", "upstream", "https://github.com/acme/app"],
    )
    .await
    .unwrap();
    set(
        dir.path(),
        &[
            ("branch.main.remote", "upstream"),
            ("branch.main.merge", "refs/pull/42/head"),
        ],
    )
    .await;

    let main = config(dir.path(), Some("main")).await;
    let login = config(dir.path(), Some("feature/login")).await;

    assert_eq!(main.remotes.len(), 1);
    assert_eq!(main.tracking.as_deref(), Some("upstream"));
    assert_eq!(main.merge.as_deref(), Some("refs/pull/42/head"));
    assert_eq!(login.tracking, None);
    assert_eq!(login.merge, None);
}

#[tokio::test]
async fn reads_the_remote_a_branch_pushes_to_as_git_does() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, ["init", "--quiet"]).await.unwrap();
    git(
        root,
        ["remote", "add", "origin", "git@github.com:acme/app.git"],
    )
    .await
    .unwrap();
    git(
        root,
        ["remote", "add", "fork", "git@github.com:dev/app.git"],
    )
    .await
    .unwrap();
    set(
        root,
        &[
            ("branch.feature/login.remote", "origin"),
            ("branch.feature/login.merge", "refs/pull/42/head"),
            (
                "branch.feature/login.pushRemote",
                "https://github.com/partner/app.git",
            ),
            ("branch.main.remote", "origin"),
        ],
    )
    .await;

    assert_eq!(
        pushes(root, Some("feature/login")).await.as_deref(),
        Some("https://github.com/partner/app.git")
    );
    assert_eq!(pushes(root, Some("main")).await.as_deref(), Some("origin"));
    assert_eq!(pushes(root, Some("feature/signup")).await, None);

    set(root, &[("remote.pushDefault", "fork")]).await;

    assert_eq!(
        pushes(root, Some("feature/login")).await.as_deref(),
        Some("https://github.com/partner/app.git")
    );
    assert_eq!(pushes(root, Some("main")).await.as_deref(), Some("fork"));
    assert_eq!(
        pushes(root, Some("feature/signup")).await.as_deref(),
        Some("fork")
    );
    assert_eq!(pushes(root, None).await.as_deref(), Some("fork"));
}
