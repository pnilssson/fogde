use std::path::Path;
use std::process::Stdio;

use serde::Deserialize;
use tokio::fs;

use crate::Error;
use crate::gh::{Gh, failure, spawn_error};

const REPOS: &str =
    "user/repos?per_page=100&sort=pushed&affiliation=owner,collaborator,organization_member";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    pub host: String,
    pub owner: String,
    pub name: String,
}

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

impl Repo {
    pub async fn cloned_in(&self, into: &Path) -> bool {
        let path = into.join(&self.name);
        if !fs::metadata(&path).await.is_ok_and(|m| m.is_dir()) {
            return false;
        }

        let Ok(output) = process::command("git")
            .arg("-C")
            .arg(&path)
            .args(["remote", "get-url", "origin"])
            .stdin(Stdio::null())
            .output()
            .await
        else {
            return false;
        };

        output.status.success()
            && remote(String::from_utf8_lossy(&output.stdout).trim())
                .is_some_and(|remote| self.matches(&remote))
    }

    fn matches(&self, other: &Self) -> bool {
        self.host.eq_ignore_ascii_case(&other.host)
            && self.owner.eq_ignore_ascii_case(&other.owner)
            && self.name.eq_ignore_ascii_case(&other.name)
    }
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

fn remote(url: &str) -> Option<Repo> {
    let (authority, path) = match url.split_once("://") {
        Some((_, rest)) => rest.split_once('/')?,
        None => url.split_once(':')?,
    };
    let host = authority.rsplit('@').next()?.split(':').next()?;
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, name) = path.split_once('/')?;
    if host.is_empty() || !segment(owner) || !segment(name) {
        return None;
    }

    Some(Repo {
        host: host.to_owned(),
        owner: owner.to_owned(),
        name: name.to_owned(),
    })
}

fn segment(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
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
    fn reads_remotes() {
        let hello = repo("github.com", "octocat", "hello-world");
        for url in [
            "https://github.com/octocat/hello-world.git",
            "https://user:secret@github.com/octocat/hello-world",
            "git@github.com:octocat/hello-world.git",
            "ssh://git@github.com:22/octocat/hello-world.git",
        ] {
            assert_eq!(remote(url), Some(hello.clone()), "{url}");
        }
        assert_eq!(remote("C:\\Users\\dev\\fogde"), None);
        assert_eq!(remote("https://github.com/octocat"), None);
    }

    #[test]
    fn matches_remotes_regardless_of_case() {
        let hello = repo("github.com", "octocat", "hello-world");

        assert!(hello.matches(&repo("GitHub.com", "OctoCat", "Hello-World")));
        assert!(!hello.matches(&repo("github.com", "fork", "hello-world")));
    }

    #[test]
    fn accepts_only_plain_names() {
        assert!(segment("fogde.app-2_x"));
        assert!(!segment(".."));
        assert!(!segment("a/b"));
        assert!(!segment("a\\b"));
        assert!(!segment(""));
    }
}
