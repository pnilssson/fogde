use protocol::github::{MergeMethod, ReviewEvent};
use serde_json::{Value, json};

use crate::Error;
use crate::api::{Request, Verb};

pub fn comment(number: u32, body: &str) -> Result<Request, Error> {
    let body = required(body, "write a comment")?;

    Ok(post(
        format!("issues/{number}/comments"),
        json!({ "body": body }),
    ))
}

pub fn review(number: u32, event: ReviewEvent, body: &str) -> Result<Request, Error> {
    let event = match event {
        ReviewEvent::Approve => "APPROVE",
        ReviewEvent::Changes => "REQUEST_CHANGES",
        ReviewEvent::Comment => "COMMENT",
    };
    let body = body.trim();
    if body.is_empty() && event == "REQUEST_CHANGES" {
        return Err(Error::Invalid("write what should change".to_owned()));
    }
    if body.is_empty() && event == "COMMENT" {
        return Err(Error::Invalid("write a comment".to_owned()));
    }

    let mut fields = json!({ "event": event });
    if !body.is_empty() {
        fields["body"] = json!(body);
    }

    Ok(post(format!("pulls/{number}/reviews"), fields))
}

pub fn merge(number: u32, method: MergeMethod, sha: &str) -> Result<Request, Error> {
    if sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::Invalid(
            "reload the pull request and try again".to_owned(),
        ));
    }

    let method = match method {
        MergeMethod::Merge => "merge",
        MergeMethod::Squash => "squash",
        MergeMethod::Rebase => "rebase",
    };

    Ok(Request {
        verb: Verb::Put,
        path: format!("pulls/{number}/merge"),
        body: Some(json!({ "merge_method": method, "sha": sha })),
    })
}

pub fn delete_branch(branch: &str) -> Result<Request, Error> {
    if !branch_name(branch) {
        return Err(Error::Invalid(format!("{branch} is not a branch name")));
    }

    Ok(Request {
        verb: Verb::Delete,
        path: format!("git/refs/heads/{}", encoded(branch)),
        body: None,
    })
}

pub fn update_branch(number: u32) -> Request {
    Request {
        verb: Verb::Put,
        path: format!("pulls/{number}/update-branch"),
        body: Some(json!({})),
    }
}

pub fn set_state(number: u32, pull: bool, open: bool) -> Request {
    let state = if open { "open" } else { "closed" };

    Request {
        verb: Verb::Patch,
        path: item(number, pull),
        body: Some(json!({ "state": state })),
    }
}

pub fn edit(number: u32, pull: bool, title: &str, body: &str) -> Result<Request, Error> {
    let title = required(title, "write a title")?;

    Ok(Request {
        verb: Verb::Patch,
        path: item(number, pull),
        body: Some(json!({ "title": title, "body": body })),
    })
}

pub fn labels(number: u32, labels: &[String]) -> Request {
    Request {
        verb: Verb::Put,
        path: format!("issues/{number}/labels"),
        body: Some(json!({ "labels": labels })),
    }
}

pub fn assignees(number: u32, add: &[String], remove: &[String]) -> Result<Vec<Request>, Error> {
    people(
        &format!("issues/{number}/assignees"),
        "assignees",
        add,
        remove,
    )
}

pub fn reviewers(number: u32, add: &[String], remove: &[String]) -> Result<Vec<Request>, Error> {
    people(
        &format!("pulls/{number}/requested_reviewers"),
        "reviewers",
        add,
        remove,
    )
}

pub fn rerun(run: u64) -> Request {
    Request {
        verb: Verb::Post,
        path: format!("actions/runs/{run}/rerun-failed-jobs"),
        body: None,
    }
}

pub fn new_issue(title: &str, body: &str) -> Result<Request, Error> {
    let title = required(title, "write a title")?;

    Ok(post(
        "issues".to_owned(),
        json!({ "title": title, "body": body }),
    ))
}

pub fn new_pull(
    title: &str,
    body: &str,
    head: &str,
    owner: Option<&str>,
    base: &str,
    draft: bool,
) -> Result<Request, Error> {
    let title = required(title, "write a title")?;
    let head = required(head, "switch to a branch first")?;
    let base = required(base, "choose a branch to merge into")?;
    if owner.is_some_and(|owner| !login(owner)) {
        return Err(Error::Invalid(
            "push the branch to a GitHub repository first".to_owned(),
        ));
    }

    let head = match owner {
        Some(owner) => format!("{owner}:{head}"),
        None => head.to_owned(),
    };

    Ok(post(
        "pulls".to_owned(),
        json!({ "title": title, "body": body, "head": head, "base": base, "draft": draft }),
    ))
}

fn branch_name(name: &str) -> bool {
    name != "@"
        && !name.starts_with('-')
        && !name.ends_with('.')
        && !name.contains("..")
        && !name.contains("@{")
        && !name.chars().any(|c| {
            c.is_ascii_control() || matches!(c, ' ' | '~' | '^' | ':' | '?' | '*' | '[' | '\\')
        })
        && name.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && !part.to_ascii_lowercase().ends_with(".lock")
        })
}

fn encoded(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn people(
    path: &str,
    field: &str,
    add: &[String],
    remove: &[String],
) -> Result<Vec<Request>, Error> {
    if let Some(name) = add.iter().chain(remove).find(|name| !login(name)) {
        return Err(Error::Invalid(format!("{name} is not a GitHub user")));
    }

    let mut requests = Vec::new();
    if !add.is_empty() {
        requests.push(post(path.to_owned(), json!({ field: add })));
    }
    if !remove.is_empty() {
        requests.push(Request {
            verb: Verb::Delete,
            path: path.to_owned(),
            body: Some(json!({ field: remove })),
        });
    }

    Ok(requests)
}

fn login(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn item(number: u32, pull: bool) -> String {
    if pull {
        format!("pulls/{number}")
    } else {
        format!("issues/{number}")
    }
}

fn post(path: String, body: Value) -> Request {
    Request {
        verb: Verb::Post,
        path,
        body: Some(body),
    }
}

fn required<'a>(text: &'a str, hint: &str) -> Result<&'a str, Error> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Error::Invalid(hint.to_owned()));
    }

    Ok(text)
}

#[cfg(test)]
mod tests;
