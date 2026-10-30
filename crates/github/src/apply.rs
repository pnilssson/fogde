use std::path::Path;
use std::process::Stdio;

use protocol::github::MergeMethod;
use protocol::repos::Repo;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::Error;
use crate::api::{Request, checked, path};
use crate::gh::{Gh, failure, spawn_error};

const READY: &str = "mutation($id: ID!) { markPullRequestReadyForReview(input: { pullRequestId: $id }) { clientMutationId } }";
const DRAFT: &str = "mutation($id: ID!) { convertPullRequestToDraft(input: { pullRequestId: $id }) { clientMutationId } }";

const AUTO: &str = "mutation($id: ID!, $method: PullRequestMergeMethod!) { enablePullRequestAutoMerge(input: { pullRequestId: $id, mergeMethod: $method }) { clientMutationId } }";
const MANUAL: &str = "mutation($id: ID!) { disablePullRequestAutoMerge(input: { pullRequestId: $id }) { clientMutationId } }";

#[derive(Deserialize)]
struct Created {
    number: u32,
}

impl Gh {
    pub async fn apply(&self, repo: &Repo, request: Request) -> Result<Vec<u8>, Error> {
        let host = checked(repo)?;
        let request = Request {
            path: path(repo, &request.path),
            ..request
        };

        self.rest(&host, &request).await
    }

    pub async fn create(&self, repo: &Repo, request: Request) -> Result<u32, Error> {
        let output = self.apply(repo, request).await?;
        let created: Created = serde_json::from_slice(&output)?;

        Ok(created.number)
    }

    pub async fn set_draft(&self, repo: &Repo, id: &str, draft: bool) -> Result<(), Error> {
        let host = checked(repo)?;
        let mutation = if draft { DRAFT } else { READY };

        self.graphql::<Value>(&host, mutation, json!({ "id": id }))
            .await
            .map(drop)
    }

    pub async fn set_auto_merge(
        &self,
        repo: &Repo,
        id: &str,
        method: Option<MergeMethod>,
    ) -> Result<(), Error> {
        let host = checked(repo)?;
        let Some(method) = method else {
            return self
                .graphql::<Value>(&host, MANUAL, json!({ "id": id }))
                .await
                .map(drop);
        };

        let method = match method {
            MergeMethod::Merge => "MERGE",
            MergeMethod::Squash => "SQUASH",
            MergeMethod::Rebase => "REBASE",
        };

        self.graphql::<Value>(&host, AUTO, json!({ "id": id, "method": method }))
            .await
            .map(drop)
    }

    pub async fn checkout_pull(&self, repo: &Repo, number: u32, dir: &Path) -> Result<(), Error> {
        let host = checked(repo)?;
        let output = self
            .command()
            .current_dir(dir)
            .args(["pr", "checkout", &number.to_string(), "--repo"])
            .arg(format!("{host}/{}/{}", repo.owner, repo.name))
            .env("GH_PROMPT_DISABLED", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "never")
            .env("GIT_ASKPASS", "")
            .env("SSH_ASKPASS_REQUIRE", "never")
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
