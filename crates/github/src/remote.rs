use std::cmp::Reverse;

use git::Remote;
use protocol::repos::Repo;

pub fn base(remotes: &[Remote]) -> Option<Repo> {
    let mut known: Vec<(&Remote, Repo)> = remotes
        .iter()
        .filter_map(|remote| Some((remote, published(&remote.url)?)))
        .collect();
    known.sort_by_key(|(remote, _)| Reverse(rank(&remote.name)));
    if let Some(index) = known
        .iter()
        .position(|(remote, _)| remote.resolved.is_some())
    {
        let (remote, repo) = known.swap_remove(index);
        return resolved(remote, repo);
    }

    let single = known.len() == 1;

    known
        .into_iter()
        .next()
        .filter(|(remote, _)| single || rank(&remote.name) > 0)
        .map(|(_, repo)| repo)
}

pub fn pushed(remotes: &[Remote], push: Option<&str>) -> Option<Repo> {
    let chosen = push
        .and_then(|name| tracked(remotes, name))
        .or_else(|| url(remotes, "origin"))
        .or_else(|| only(remotes))?;

    published(chosen)
}

pub fn signed(remotes: Vec<Remote>, hosts: &[String]) -> Vec<Remote> {
    let on = |remote: &Remote| {
        published(&remote.url).is_some_and(|repo| {
            hosts
                .iter()
                .any(|host| host.eq_ignore_ascii_case(&repo.host))
        })
    };
    if !remotes.iter().any(on) {
        return remotes;
    }

    remotes.into_iter().filter(on).collect()
}

pub fn pulled(remotes: &[Remote], tracking: Option<&str>, merge: Option<&str>) -> Option<u32> {
    let number = merge?
        .strip_prefix("refs/pull/")?
        .strip_suffix("/head")?
        .parse()
        .ok()?;
    let from = published(tracked(remotes, tracking?)?)?;

    same(&from, &base(remotes)?).then_some(number)
}

pub(crate) fn same(a: &Repo, b: &Repo) -> bool {
    a.host.eq_ignore_ascii_case(&b.host)
        && a.owner.eq_ignore_ascii_case(&b.owner)
        && a.name.eq_ignore_ascii_case(&b.name)
}

fn tracked<'a>(remotes: &'a [Remote], name: &'a str) -> Option<&'a str> {
    url(remotes, name).or_else(|| remote(name).map(|_| name))
}

fn rank(name: &str) -> u8 {
    match name.to_ascii_lowercase().as_str() {
        "upstream" => 3,
        "github" => 2,
        "origin" => 1,
        _ => 0,
    }
}

fn resolved(remote: &Remote, repo: Repo) -> Option<Repo> {
    let value = remote.resolved.as_deref()?;
    if value == "base" {
        return Some(repo);
    }

    let (owner, name) = value.split_once('/')?;
    if !segment(owner) || !segment(name) {
        return None;
    }

    Some(Repo {
        owner: owner.to_owned(),
        name: name.to_owned(),
        ..repo
    })
}

fn only(remotes: &[Remote]) -> Option<&str> {
    match remotes {
        [remote] => Some(&remote.url),
        _ => None,
    }
}

fn url<'a>(remotes: &'a [Remote], name: &str) -> Option<&'a str> {
    remotes
        .iter()
        .find(|remote| remote.name == name)
        .map(|remote| remote.url.as_str())
}

fn published(url: &str) -> Option<Repo> {
    let repo = remote(url)?;

    Some(Repo {
        host: repo.host.to_ascii_lowercase(),
        ..repo
    })
}

pub(crate) fn remote(url: &str) -> Option<Repo> {
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

pub(crate) fn segment(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests;
