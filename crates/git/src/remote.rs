use std::cmp::Reverse;
use std::path::Path;

use crate::run::git;

const KEYS: &str = r"^(remote\..*\.(url|gh-resolved)|url\..*\.insteadof|remote\.pushdefault|branch\..*\.(remote|pushremote|merge))$";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub resolved: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub remotes: Vec<Remote>,
    pub tracking: Option<String>,
    pub merge: Option<String>,
    pub push: Option<String>,
}

pub async fn config(root: &Path, branch: Option<&str>) -> Config {
    let Ok(output) = git(root, ["config", "--null", "--get-regexp", KEYS]).await else {
        return Config::default();
    };

    parse(&String::from_utf8_lossy(&output), branch)
}

pub async fn remotes(root: &Path) -> Vec<Remote> {
    config(root, None).await.remotes
}

fn parse(output: &str, branch: Option<&str>) -> Config {
    let pairs: Vec<(&str, &str)> = output
        .split('\0')
        .filter_map(|entry| {
            let (key, value) = entry.split_once('\n')?;

            Some((key, value.trim()))
        })
        .collect();
    let last = |key: &str| {
        pairs
            .iter()
            .rev()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| *value)
            .filter(|value| !value.is_empty() && *value != ".")
            .map(str::to_owned)
    };
    let field = |name: &str| branch.and_then(|branch| last(&format!("branch.{branch}.{name}")));

    let tracking = field("remote");
    let push = field("pushremote")
        .or_else(|| last("remote.pushdefault"))
        .or_else(|| tracking.clone());

    Config {
        remotes: listed(&pairs),
        merge: field("merge"),
        push,
        tracking,
    }
}

fn listed(pairs: &[(&str, &str)]) -> Vec<Remote> {
    let entries: Vec<(&str, &str, &str, &str)> = pairs
        .iter()
        .filter_map(|(key, value)| {
            let (section, key) = key.split_once('.')?;
            let (name, field) = key.rsplit_once('.')?;

            Some((section, name, field, *value))
        })
        .collect();
    let values = |section: &'static str, field: &'static str| {
        entries
            .iter()
            .filter(move |entry| entry.0 == section && entry.2 == field)
            .map(|entry| (entry.1, entry.3))
    };
    let resolved = |name: &str| {
        values("remote", "gh-resolved")
            .rev()
            .find(|(remote, _)| *remote == name)
            .map(|(_, value)| value.to_owned())
    };

    let mut remotes: Vec<Remote> = Vec::new();
    for (name, url) in values("remote", "url") {
        if remotes.iter().any(|remote| remote.name == name) {
            continue;
        }

        remotes.push(Remote {
            name: name.to_owned(),
            url: rewrite(url, values("url", "insteadof")),
            resolved: resolved(name),
        });
    }

    remotes
}

fn rewrite<'a>(url: &str, rules: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let rule = rules
        .filter(|(_, prefix)| url.starts_with(prefix))
        .min_by_key(|(_, prefix)| Reverse(prefix.len()));

    match rule {
        Some((base, prefix)) => format!("{base}{}", &url[prefix.len()..]),
        None => url.to_owned(),
    }
}

#[cfg(test)]
mod tests;
