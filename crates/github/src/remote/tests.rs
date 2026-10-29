use super::*;

fn remotes(entries: &[(&str, &str)]) -> Vec<Remote> {
    entries
        .iter()
        .map(|(name, url)| Remote {
            name: (*name).to_owned(),
            url: (*url).to_owned(),
            resolved: None,
        })
        .collect()
}

fn resolve(mut remotes: Vec<Remote>, name: &str, value: &str) -> Vec<Remote> {
    for remote in &mut remotes {
        if remote.name == name {
            remote.resolved = Some(value.to_owned());
        }
    }

    remotes
}

fn named(repo: Option<Repo>) -> Option<String> {
    repo.map(|repo| format!("{}/{}/{}", repo.host, repo.owner, repo.name))
}

#[test]
fn reads_remotes() {
    for url in [
        "https://github.com/dev/app.git",
        "https://user:secret@github.com/dev/app",
        "git@github.com:dev/app.git",
        "ssh://git@github.com:22/dev/app.git",
    ] {
        assert_eq!(
            remote(url),
            Some(Repo {
                host: "github.com".to_owned(),
                owner: "dev".to_owned(),
                name: "app".to_owned(),
            }),
            "{url}"
        );
    }
    assert_eq!(remote(r"C:\Users\dev\app"), None);
    assert_eq!(remote("https://github.com/dev"), None);
}

#[test]
fn takes_the_base_repo_as_gh_does() {
    let fork = remotes(&[
        ("origin", "git@github.com:dev/app.git"),
        ("upstream", "https://Acme.ghe.com/acme/app"),
    ]);

    assert_eq!(named(base(&fork)).as_deref(), Some("acme.ghe.com/acme/app"));
    assert_eq!(
        named(base(&remotes(&[
            ("mirror", "git@github.com:dev/mirror.git"),
            ("origin", "git@github.com:acme/app.git"),
        ])))
        .as_deref(),
        Some("github.com/acme/app")
    );
    assert_eq!(
        named(base(&remotes(&[("fork", "git@github.com:dev/app.git")]))).as_deref(),
        Some("github.com/dev/app")
    );
}

#[test]
fn skips_remotes_that_are_not_repositories_as_gh_does() {
    let local = remotes(&[
        ("origin", "/repos/app"),
        ("fork", "git@github.com:dev/app.git"),
    ]);

    assert_eq!(named(base(&local)).as_deref(), Some("github.com/dev/app"));
    assert_eq!(
        named(base(&resolve(
            remotes(&[
                ("upstream", "/repos/app"),
                ("origin", "git@github.com:dev/app.git"),
                ("mirror", "git@github.com:acme/app.git"),
            ]),
            "mirror",
            "base"
        )))
        .as_deref(),
        Some("github.com/acme/app")
    );
}

#[test]
fn honours_the_default_repo_set_with_gh() {
    let fork = remotes(&[
        ("origin", "git@github.com:dev/app.git"),
        ("upstream", "https://github.com/acme/app"),
    ]);

    assert_eq!(
        named(base(&resolve(fork.clone(), "origin", "base"))).as_deref(),
        Some("github.com/dev/app")
    );
    assert_eq!(
        named(base(&resolve(fork.clone(), "origin", "partner/app"))).as_deref(),
        Some("github.com/partner/app")
    );
    assert_eq!(
        named(base(&resolve(
            remotes(&[
                ("fork", "git@github.com:dev/app.git"),
                ("mirror", "git@github.com:acme/app.git"),
            ]),
            "mirror",
            "base"
        )))
        .as_deref(),
        Some("github.com/acme/app")
    );
    assert_eq!(base(&resolve(fork, "origin", "../x")), None);
}

#[test]
fn takes_the_pushed_repo_from_the_push_remote() {
    let fork = remotes(&[
        ("origin", "git@github.com:dev/app.git"),
        ("upstream", "https://github.com/acme/app"),
    ]);

    assert_eq!(
        named(pushed(&fork, Some("upstream"))).as_deref(),
        Some("github.com/acme/app")
    );
    assert_eq!(
        named(pushed(&fork, None)).as_deref(),
        Some("github.com/dev/app")
    );
    assert_eq!(
        named(pushed(&fork, Some("https://github.com/partner/app.git"))).as_deref(),
        Some("github.com/partner/app")
    );
}

#[test]
fn never_guesses_between_other_remotes() {
    let others = remotes(&[
        ("fork", "git@github.com:dev/app.git"),
        ("mirror", "git@github.com:acme/app.git"),
    ]);

    assert_eq!(base(&others), None);
    assert_eq!(pushed(&others, None), None);
    assert_eq!(base(&[]), None);
    assert_eq!(base(&remotes(&[("origin", "/repos/app")])), None);
}

#[test]
fn accepts_only_plain_names() {
    assert!(segment("my.app-2_x"));
    assert!(!segment(".."));
    assert!(!segment("a/b"));
    assert!(!segment(r"a\b"));
    assert!(!segment(""));
}

#[test]
fn reads_the_pull_request_a_branch_was_checked_out_from() {
    let fork = remotes(&[
        ("origin", "git@github.com:dev/app.git"),
        ("upstream", "https://github.com/acme/app"),
    ]);

    assert_eq!(
        pulled(&fork, Some("upstream"), Some("refs/pull/42/head")),
        Some(42)
    );
    assert_eq!(
        pulled(
            &fork,
            Some("https://github.com/acme/app.git"),
            Some("refs/pull/42/head")
        ),
        Some(42)
    );
    assert_eq!(
        pulled(&fork, Some("origin"), Some("refs/pull/42/head")),
        None
    );
    assert_eq!(
        pulled(&fork, Some("upstream"), Some("refs/heads/feature/login")),
        None
    );
    assert_eq!(
        pulled(&fork, Some("upstream"), Some("refs/pull/x/head")),
        None
    );
    assert_eq!(pulled(&fork, None, Some("refs/pull/42/head")), None);
    assert_eq!(pulled(&fork, Some("upstream"), None), None);
}

#[test]
fn keeps_remotes_on_signed_in_hosts_as_gh_does() {
    let mixed = remotes(&[
        ("origin", "git@github.com:dev/app.git"),
        ("upstream", "https://gitlab.com/acme/app.git"),
        ("local", "/repos/app"),
    ]);
    let hosts = ["GitHub.com".to_owned()];

    assert_eq!(
        named(base(&signed(mixed.clone(), &hosts))).as_deref(),
        Some("github.com/dev/app")
    );
    assert_eq!(signed(mixed.clone(), &[]), mixed);
    assert_eq!(signed(mixed.clone(), &["acme.ghe.com".to_owned()]), mixed);
}
