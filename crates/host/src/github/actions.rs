use github::Request;
use protocol::github::{
    AddComment, CheckoutPull, CheckoutPullParams, CommentParams, DraftParams, EditItem, EditParams,
    ItemParams, LabelsParams, MergeParams, MergePull, NewIssue, NewIssueParams, NewPull,
    NewPullParams, PeopleParams, RerunFailed, RerunParams, ReviewParams, SetAssignees, SetDraft,
    SetLabels, SetReviewers, SetState, StateParams, SubmitReview, UpdateBranch,
};
use protocol::repos::Repo;
use protocol::rpc::{Failure, SERVER_ERROR};

use crate::checkouts::{published, root};
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<AddComment>();
    router.add::<SubmitReview>();
    router.add::<MergePull>();
    router.add::<UpdateBranch>();
    router.add::<SetDraft>();
    router.add::<SetState>();
    router.add::<EditItem>();
    router.add::<SetLabels>();
    router.add::<SetAssignees>();
    router.add::<SetReviewers>();
    router.add::<RerunFailed>();
    router.add::<NewIssue>();
    router.add::<NewPull>();
    router.add::<CheckoutPull>();
}

async fn apply(session: &Session, repo: &Repo, requests: Vec<Request>) -> Result<(), Failure> {
    let gh = session.gh().await?;
    for request in requests {
        gh.apply(repo, request).await.map_err(failure)?;
    }

    Ok(())
}

impl Handler for AddComment {
    async fn handle(session: &Session, params: CommentParams) -> Result<(), Failure> {
        let request = github::comment(params.number, &params.body).map_err(failure)?;

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for SubmitReview {
    async fn handle(session: &Session, params: ReviewParams) -> Result<(), Failure> {
        let request = github::review(params.number, params.event, &params.body).map_err(failure)?;

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for MergePull {
    async fn handle(session: &Session, params: MergeParams) -> Result<(), Failure> {
        let merge = github::merge(params.number, params.method, &params.sha).map_err(failure)?;
        let mut delete = None;
        if params.delete {
            let pull = session
                .gh()
                .await?
                .pull(&params.repo, params.number)
                .await
                .map_err(failure)?
                .pull
                .ok_or_else(|| failure(format!("#{} is not a pull request", params.number)))?;
            if !pull.cross {
                delete = Some(github::delete_branch(&pull.head).map_err(failure)?);
            }
        }

        apply(session, &params.repo, vec![merge]).await?;

        let Some(delete) = delete else {
            return Ok(());
        };

        apply(session, &params.repo, vec![delete])
            .await
            .map_err(|error| {
                Failure::new(
                    SERVER_ERROR,
                    format!("merged, but the branch stays: {}", error.message),
                )
            })
    }
}

impl Handler for UpdateBranch {
    async fn handle(session: &Session, params: ItemParams) -> Result<(), Failure> {
        let request = github::update_branch(params.number);

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for SetDraft {
    async fn handle(session: &Session, params: DraftParams) -> Result<(), Failure> {
        session
            .gh()
            .await?
            .set_draft(&params.repo, &params.id, params.draft)
            .await
            .map_err(failure)
    }
}

impl Handler for SetState {
    async fn handle(session: &Session, params: StateParams) -> Result<(), Failure> {
        let request = github::set_state(params.number, params.pull, params.open);

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for EditItem {
    async fn handle(session: &Session, params: EditParams) -> Result<(), Failure> {
        let request = github::edit(params.number, params.pull, &params.title, &params.body)
            .map_err(failure)?;

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for SetLabels {
    async fn handle(session: &Session, params: LabelsParams) -> Result<(), Failure> {
        let request = github::labels(params.number, &params.labels);

        apply(session, &params.repo, vec![request]).await
    }
}

impl Handler for SetAssignees {
    async fn handle(session: &Session, params: PeopleParams) -> Result<(), Failure> {
        let requests =
            github::assignees(params.number, &params.add, &params.remove).map_err(failure)?;

        apply(session, &params.repo, requests).await
    }
}

impl Handler for SetReviewers {
    async fn handle(session: &Session, params: PeopleParams) -> Result<(), Failure> {
        let requests =
            github::reviewers(params.number, &params.add, &params.remove).map_err(failure)?;

        apply(session, &params.repo, requests).await
    }
}

impl Handler for RerunFailed {
    async fn handle(session: &Session, params: RerunParams) -> Result<(), Failure> {
        apply(session, &params.repo, vec![github::rerun(params.run)]).await
    }
}

impl Handler for NewIssue {
    async fn handle(session: &Session, params: NewIssueParams) -> Result<u32, Failure> {
        let request = github::new_issue(&params.title, &params.body).map_err(failure)?;

        session
            .gh()
            .await?
            .create(&params.repo, request)
            .await
            .map_err(failure)
    }
}

impl Handler for NewPull {
    async fn handle(session: &Session, params: NewPullParams) -> Result<u32, Failure> {
        let root = root(&params.path).await?;
        let status = git::status(&root).await.map_err(failure)?;
        let Some(head) = status.branch else {
            return Err(failure("switch to a branch before opening a pull request"));
        };
        github::new_pull(
            &params.title,
            &params.body,
            &head,
            None,
            &params.base,
            params.draft,
        )
        .map_err(failure)?;

        if status.upstream.is_none() || status.ahead > 0 {
            git::push(&root).await.map_err(failure)?;
        }

        let owner = published(&root, Some(&head), &session.shared.hosts())
            .await
            .owner;
        let owner = owner.filter(|owner| !owner.eq_ignore_ascii_case(&params.repo.owner));
        let request = github::new_pull(
            &params.title,
            &params.body,
            &head,
            owner.as_deref(),
            &params.base,
            params.draft,
        )
        .map_err(failure)?;

        session
            .gh()
            .await?
            .create(&params.repo, request)
            .await
            .map_err(failure)
    }
}

impl Handler for CheckoutPull {
    async fn handle(session: &Session, params: CheckoutPullParams) -> Result<String, Failure> {
        let root = root(&params.path).await?;
        let gh = session.gh().await?;
        if !params.worktree {
            gh.checkout_pull(&params.repo, params.number, &root)
                .await
                .map_err(failure)?;

            return Ok(root.display().to_string());
        }

        let tree =
            crate::worktrees::for_pull(&session.shared.worktrees(), &root, params.number).await?;
        if let Err(error) = gh
            .checkout_pull(&params.repo, params.number, &tree.path)
            .await
        {
            if tree.fresh {
                git::discard_worktree(&tree.main, &tree.path).await;
            }

            return Err(failure(error));
        }

        Ok(tree.path.display().to_string())
    }
}
