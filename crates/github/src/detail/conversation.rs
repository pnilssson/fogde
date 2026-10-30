use protocol::github::{Comment, Thread, Verdict};
use serde::Deserialize;

use crate::list::{Login, Nodes};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawComment {
    id: String,
    url: String,
    created_at: Option<String>,
    submitted_at: Option<String>,
    state: Option<String>,
    #[serde(rename = "bodyHTML")]
    html: String,
    author: Option<Login>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawThread {
    is_resolved: bool,
    is_outdated: bool,
    path: String,
    line: Option<u32>,
    original_line: Option<u32>,
    comments: Nodes<RawComment>,
}

pub(super) fn thread(raw: RawThread) -> Option<Thread> {
    if raw.is_resolved {
        return None;
    }

    let comments: Vec<Comment> = raw.comments.items().filter_map(comment).collect();
    if comments.is_empty() {
        return None;
    }

    Some(Thread {
        path: raw.path,
        line: raw.line.or(raw.original_line),
        outdated: raw.is_outdated,
        comments,
    })
}

pub(super) fn comment(raw: RawComment) -> Option<Comment> {
    let verdict = match raw.state.as_deref() {
        None => None,
        Some("APPROVED") => Some(Verdict::Approved),
        Some("CHANGES_REQUESTED") => Some(Verdict::Changes),
        Some("DISMISSED") => Some(Verdict::Dismissed),
        Some("COMMENTED") if !raw.html.trim().is_empty() => Some(Verdict::Commented),
        Some(_) => return None,
    };

    Some(Comment {
        id: raw.id,
        author: raw.author.map(|author| author.login),
        html: raw.html,
        created: raw.created_at.or(raw.submitted_at)?,
        url: raw.url,
        verdict,
    })
}
