use std::process::{Output, Stdio};

use protocol::repos::Repo;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::Error;
use crate::gh::{Gh, failure, spawn_error};
use crate::remote::segment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub verb: Verb,
    pub path: String,
    pub body: Option<Value>,
}

#[derive(Deserialize)]
struct Reply<T> {
    data: T,
}

#[derive(Deserialize)]
struct Rejection {
    message: String,
    #[serde(default)]
    errors: Vec<Value>,
    status: Option<String>,
}

impl Gh {
    pub(crate) async fn graphql<T: DeserializeOwned>(
        &self,
        host: &str,
        query: &str,
        variables: Value,
    ) -> Result<T, Error> {
        let body = json!({ "query": query, "variables": variables });
        let output = self
            .send(&["api", "graphql", "--hostname", host], Some(&body))
            .await?;
        if !output.status.success() {
            return Err(rejected(&output));
        }

        let reply: Reply<T> = serde_json::from_slice(&output.stdout)?;

        Ok(reply.data)
    }

    pub(crate) async fn rest(&self, host: &str, request: &Request) -> Result<Vec<u8>, Error> {
        let verb = match request.verb {
            Verb::Post => "POST",
            Verb::Put => "PUT",
            Verb::Patch => "PATCH",
            Verb::Delete => "DELETE",
        };
        let output = self
            .send(
                &["api", "--hostname", host, "--method", verb, &request.path],
                request.body.as_ref(),
            )
            .await?;
        if !output.status.success() {
            return Err(rejected(&output));
        }

        Ok(output.stdout)
    }

    pub(crate) async fn read(&self, host: &str, path: &str) -> Result<Vec<u8>, Error> {
        let output = self
            .send(
                &["api", "--hostname", host, "--allow-escape-sequences", path],
                None,
            )
            .await?;
        if !output.status.success() {
            return Err(rejected(&output));
        }

        Ok(output.stdout)
    }

    async fn send(&self, args: &[&str], body: Option<&Value>) -> Result<Output, Error> {
        let input = body.map(serde_json::to_vec).transpose()?;
        let mut command = self.command();
        command.args(args);
        if input.is_some() {
            command.args(["--input", "-"]);
        }

        command.env("GH_PROMPT_DISABLED", "1").env("NO_COLOR", "1");
        match input {
            Some(input) => process::feed(&mut command, &input).await,
            None => command.stdin(Stdio::null()).output().await,
        }
        .map_err(spawn_error)
    }
}

pub(crate) fn checked(repo: &Repo) -> Result<String, Error> {
    let host = crate::host(&repo.host)?;
    if !segment(&repo.owner) || !segment(&repo.name) {
        return Err(Error::Repo(format!("{}/{}", repo.owner, repo.name)));
    }

    Ok(host)
}

pub(crate) fn path(repo: &Repo, rest: &str) -> String {
    format!("repos/{}/{}/{rest}", repo.owner, repo.name)
}

fn rejected(output: &Output) -> Error {
    let Ok(rejection) = serde_json::from_slice::<Rejection>(&output.stdout) else {
        return graphql_error(&output.stdout).unwrap_or_else(|| failure(&output.stderr));
    };
    if rejection.status.as_deref() == Some("410") {
        return Error::Gh("GitHub no longer keeps it".to_owned());
    }

    let detail = rejection.errors.iter().find_map(|error| match error {
        Value::String(text) => Some(text.clone()),
        Value::Object(fields) => fields.get("message")?.as_str().map(str::to_owned),
        _ => None,
    });

    Error::Gh(match detail {
        Some(detail) => format!("{}: {detail}", rejection.message),
        None => rejection.message,
    })
}

fn graphql_error(stdout: &[u8]) -> Option<Error> {
    let reply: Value = serde_json::from_slice(stdout).ok()?;
    let message = reply.get("errors")?.get(0)?.get("message")?.as_str()?;

    Some(Error::Gh(message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(stdout: &str, stderr: &str) -> Output {
        Output {
            status: std::process::ExitStatus::default(),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn explains_rejections_with_github_messages() {
        let rest = output(
            r#"{"message":"Validation Failed","errors":[{"resource":"Issue","code":"custom","field":"title","message":"title is too long"}],"documentation_url":"https://docs.github.com/rest","status":"422"}"#,
            "gh: Validation Failed (HTTP 422)",
        );
        let merge = output(
            r#"{"message":"Pull Request is not mergeable","documentation_url":"https://docs.github.com/rest","status":"405"}"#,
            "gh: Pull Request is not mergeable (HTTP 405)",
        );
        let graphql = output(
            r#"{"data":{"repository":null},"errors":[{"type":"NOT_FOUND","path":["repository"],"message":"Could not resolve to a Repository with the name 'acme/missing'."}]}"#,
            "gh: Could not resolve to a Repository with the name 'acme/missing'.",
        );

        assert_eq!(
            rejected(&rest).to_string(),
            "the GitHub CLI failed: Validation Failed: title is too long"
        );
        assert_eq!(
            rejected(&merge).to_string(),
            "the GitHub CLI failed: Pull Request is not mergeable"
        );
        assert_eq!(
            rejected(&graphql).to_string(),
            "the GitHub CLI failed: Could not resolve to a Repository with the name 'acme/missing'."
        );
        assert_eq!(
            rejected(&output(
                "",
                "gh: To get started with GitHub CLI, please run:  gh auth login"
            ))
            .to_string(),
            "the GitHub CLI failed: To get started with GitHub CLI, please run:  gh auth login"
        );
        assert_eq!(
            rejected(&output(
                r#"{"message":"Server Error","documentation_url":"https://docs.github.com/rest/actions/workflow-jobs#download-job-logs-for-a-workflow-run","status":"410"}"#,
                "gh: Server Error (HTTP 410)",
            ))
            .to_string(),
            "the GitHub CLI failed: GitHub no longer keeps it"
        );
    }

    #[test]
    fn refuses_repos_that_would_change_the_request_path() {
        let repo = |owner: &str, name: &str| Repo {
            host: "github.com".to_owned(),
            owner: owner.to_owned(),
            name: name.to_owned(),
        };

        assert_eq!(checked(&repo("acme", "app")).unwrap(), "github.com");
        assert!(checked(&repo("acme", "../orgs")).is_err());
        assert!(checked(&repo("acme", "app?per_page=1")).is_err());
        assert!(checked(&repo("acme/x", "app")).is_err());
        assert!(
            checked(&Repo {
                host: "github.com --paginate".to_owned(),
                ..repo("acme", "app")
            })
            .is_err()
        );
    }
}
