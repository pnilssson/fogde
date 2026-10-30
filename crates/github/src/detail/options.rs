use protocol::github::RepoOptions;
use protocol::repos::Repo;
use serde::Deserialize;
use serde_json::json;

use super::Reply;
use crate::Error;
use crate::api::checked;
use crate::gh::Gh;
use crate::list::{Login, Nodes, RawLabel, labels, logins};

const OPTIONS: &str = include_str!("../queries/options.graphql");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionsRepo {
    default_branch_ref: Option<Branch>,
    labels: Option<Nodes<RawLabel>>,
    assignable_users: Option<Nodes<Login>>,
}

#[derive(Deserialize)]
struct Branch {
    name: String,
}

impl Gh {
    pub async fn options(&self, repo: &Repo) -> Result<RepoOptions, Error> {
        let host = checked(repo)?;
        let variables = json!({ "owner": repo.owner, "name": repo.name });
        let reply: Reply<OptionsRepo> = self.graphql(&host, OPTIONS, variables).await?;
        let raw = reply
            .repository
            .ok_or_else(|| Error::Gh(format!("{}/{} was not found", repo.owner, repo.name)))?;

        Ok(RepoOptions {
            labels: labels(raw.labels),
            people: logins(raw.assignable_users),
            branch: raw.default_branch_ref.map(|branch| branch.name),
        })
    }
}
