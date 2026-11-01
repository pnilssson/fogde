use protocol::github::{Comment, Detail, FileStat, Linked, PullDetail};

use super::checks::check;
use super::{Page, RawItem, Rules, comment, merge, method, thread};
use crate::list::{Nodes, labels, logins, review, state};

pub(super) fn detail(mut raw: RawItem, rules: Rules) -> Detail {
    let reviews = raw.reviews.take().map(Nodes::items).into_iter().flatten();
    let mut comments: Vec<Comment> = raw
        .comments
        .take()
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .chain(reviews)
        .filter_map(comment)
        .collect();
    comments.sort_by(|a, b| a.created.cmp(&b.created));

    let linked = raw
        .closing_issues_references
        .take()
        .map(Nodes::items)
        .into_iter()
        .flatten()
        .map(|link| Linked {
            number: link.number,
            title: link.title,
            url: link.url,
            state: state(&link.state),
        })
        .collect();

    let pull = pull_detail(&mut raw, rules);

    Detail {
        id: raw.id,
        number: raw.number,
        title: raw.title,
        url: raw.url,
        state: state(&raw.state),
        author: raw.author.map(|author| author.login),
        created: raw.created_at,
        body: raw.body,
        html: raw.html,
        labels: labels(raw.labels),
        assignees: logins(raw.assignees),
        editable: raw.viewer_can_update,
        comments,
        linked,
        pull,
    }
}

pub(super) fn pull_detail(raw: &mut RawItem, rules: Rules) -> Option<PullDetail> {
    let head = raw.head_ref_name.take()?;
    let checks = raw
        .commits
        .take()
        .and_then(|commits| commits.items().next())
        .and_then(|node| node.commit.status_check_rollup)
        .map(|rollup| rollup.contexts.items().filter_map(check).collect())
        .unwrap_or_default();

    Some(PullDetail {
        draft: raw.is_draft.unwrap_or(false),
        head,
        owner: raw.head_repository_owner.take().map(|owner| owner.login),
        sha: raw.head_ref_oid.take().unwrap_or_default(),
        base: raw.base_ref_name.take().unwrap_or_default(),
        cross: raw.is_cross_repository.unwrap_or(false),
        review: raw.review_decision.as_deref().and_then(review),
        merge: merge(raw.merge_state_status.as_deref().unwrap_or_default()),
        methods: rules.methods,
        automerge: rules.automerge,
        auto: raw
            .auto_merge_request
            .take()
            .and_then(|request| method(&request.merge_method)),
        reviewers: raw
            .review_requests
            .take()
            .map(Nodes::items)
            .into_iter()
            .flatten()
            .filter_map(|request| request.requested_reviewer?.login)
            .collect(),
        checks,
        threads: raw
            .review_threads
            .take()
            .map(Page::items)
            .into_iter()
            .flatten()
            .filter_map(thread)
            .collect(),
        files: raw
            .files
            .take()
            .map(Nodes::items)
            .into_iter()
            .flatten()
            .map(|file| FileStat {
                path: file.path,
                added: file.additions,
                removed: file.deletions,
            })
            .collect(),
        added: raw.additions.unwrap_or(0),
        removed: raw.deletions.unwrap_or(0),
        changed: raw.changed_files.unwrap_or(0),
    })
}
