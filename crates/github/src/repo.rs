use std::path::Path;
use std::process::Stdio;

use protocol::repos::Repo;
use serde::Deserialize;
use tokio::fs;

use crate::Error;
use crate::gh::{Gh, failure, spawn_error};
use crate::remote::{remote, same, segment};

const REPOS: &str =
    "user/repos?per_page=100&sort=pushed&affiliation=owner,collaborator,organization_member";

#[derive(Deserialize)]
struct Entry {
    name: String,
    archived: bool,
    owner: Owner,
}

#[derive(Deserialize)]
struct Owner {
    login: String,
}

impl Gh {
    pub async fn repos(&self, host: &str) -> Result<Vec<Repo>, Error> {
        let output = self
            .run(&["api", "--paginate", "--hostname", host, REPOS])
            .await?;
        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        pages(host, &output.stdout)
    }

    pub async fn clone_repo(&self, repo: &Repo, into: &Path) -> Result<(), Error> {
        if !segment(&repo.owner) || !segment(&repo.name) {
            return Err(Error::Repo(format!("{}/{}", repo.owner, repo.name)));
        }
        let path = into.join(&repo.name);
        if fs::try_exists(&path).await? {
            return Err(Error::Exists(path));
        }

        let output = self
            .command()
            .args(["repo", "clone"])
            .arg(format!("{}/{}/{}", repo.host, repo.owner, repo.name))
            .arg(&path)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(spawn_error)?;
        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        Ok(())
    }
}

pub async fn cloned(repo: &Repo, into: &Path) -> bool {
    let path = into.join(&repo.name);
    if !fs::metadata(&path).await.is_ok_and(|m| m.is_dir()) {
        return false;
    }

    git::remotes(&path)
        .await
        .into_iter()
        .find(|found| found.name == "origin")
        .and_then(|origin| remote(&origin.url))
        .is_some_and(|origin| same(repo, &origin))
}

fn pages(host: &str, output: &[u8]) -> Result<Vec<Repo>, Error> {
    let mut repos = Vec::new();
    for page in serde_json::Deserializer::from_slice(output).into_iter::<Vec<Entry>>() {
        repos.extend(visible(host, page?));
    }

    Ok(repos)
}

fn visible(host: &str, entries: Vec<Entry>) -> impl Iterator<Item = Repo> {
    entries
        .into_iter()
        .filter(|entry| !entry.archived)
        .map(move |entry| Repo {
            host: host.to_owned(),
            owner: entry.owner.login,
            name: entry.name,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(host: &str, owner: &str, name: &str) -> Repo {
        Repo {
            host: host.to_owned(),
            owner: owner.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn reads_every_page_and_skips_archived_repos() {
        let json = br#"[{"name":"hello-world","full_name":"octocat/hello-world","private":false,"owner":{"login":"octocat","type":"User"},"fork":false,"archived":false,"pushed_at":"2026-10-17T21:39:30Z"},{"name":"notes","full_name":"octocat/notes","private":true,"owner":{"login":"octocat","type":"User"},"fork":false,"archived":true,"pushed_at":"2026-10-16T21:06:45Z"}][{"name":"blog","full_name":"octocat/blog","private":true,"owner":{"login":"octocat","type":"User"},"fork":false,"archived":false,"pushed_at":"2026-10-15T08:12:03Z"}]"#;

        assert_eq!(
            pages("github.com", json).unwrap(),
            [
                repo("github.com", "octocat", "hello-world"),
                repo("github.com", "octocat", "blog"),
            ]
        );
    }

    #[test]
    fn matches_remotes_regardless_of_case() {
        let hello = repo("github.com", "octocat", "hello-world");

        assert!(same(&hello, &repo("GitHub.com", "OctoCat", "Hello-World")));
        assert!(!same(&hello, &repo("github.com", "fork", "hello-world")));
    }
}
