use std::path::Path;
use std::process::Stdio;

use protocol::repos::Repo;
use serde::Deserialize;
use tokio::fs;
use tokio::process::Command;

use crate::Error;
use crate::gh::{Gh, failure};
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

        let output = cloning(self.command(), repo, &path)
            .output()
            .await
            .map_err(process::not_found(Error::Missing))?;
        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        Ok(())
    }
}

fn cloning(mut gh: Command, repo: &Repo, path: &Path) -> Command {
    gh.args(["repo", "clone"])
        .arg(format!("{}/{}/{}", repo.host, repo.owner, repo.name))
        .arg(path)
        .envs(process::UNPROMPTED)
        .stdin(Stdio::null());

    gh
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
    use std::ffi::OsStr;

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
    fn clones_without_prompting_for_credentials() {
        let clone = cloning(
            Command::new("gh"),
            &repo("github.com", "octocat", "hello-world"),
            Path::new("/repos/hello-world"),
        );
        let env: Vec<(&OsStr, Option<&OsStr>)> = clone.as_std().get_envs().collect();

        for (name, value) in process::UNPROMPTED {
            assert!(
                env.contains(&(OsStr::new(name), Some(OsStr::new(value)))),
                "missing {name}"
            );
        }
    }

    #[test]
    fn matches_remotes_regardless_of_case() {
        let hello = repo("github.com", "octocat", "hello-world");

        assert!(same(&hello, &repo("GitHub.com", "OctoCat", "Hello-World")));
        assert!(!same(&hello, &repo("github.com", "fork", "hello-world")));
    }
}
