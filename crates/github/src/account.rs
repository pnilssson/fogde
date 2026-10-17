use std::collections::HashMap;

use serde::Deserialize;

use crate::Error;
use crate::gh::{Gh, failure};

pub const GITHUB: &str = "github.com";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub host: String,
    pub login: String,
}

#[derive(Deserialize)]
struct Status {
    hosts: HashMap<String, Vec<Entry>>,
}

#[derive(Deserialize)]
struct Entry {
    state: String,
    host: String,
    login: String,
}

impl Gh {
    pub async fn accounts(&self) -> Result<Vec<Account>, Error> {
        let output = self
            .run(&["auth", "status", "--active", "--json", "hosts"])
            .await?;
        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        parse(&output.stdout)
    }
}

fn parse(json: &[u8]) -> Result<Vec<Account>, Error> {
    let status: Status = serde_json::from_slice(json)?;
    let mut accounts: Vec<Account> = status
        .hosts
        .into_values()
        .flatten()
        .filter(|entry| entry.state == "success")
        .map(|entry| Account {
            host: entry.host,
            login: entry.login,
        })
        .collect();
    accounts.sort_by(|a, b| (a.host != GITHUB, &a.host).cmp(&(b.host != GITHUB, &b.host)));

    Ok(accounts)
}

pub fn host(input: &str) -> Result<String, Error> {
    let host = input.trim();
    let host = host
        .strip_prefix("https://")
        .or_else(|| host.strip_prefix("http://"))
        .unwrap_or(host)
        .trim_end_matches('/')
        .to_ascii_lowercase();

    let valid = !host.is_empty()
        && host.contains('.')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !valid {
        return Err(Error::Host(input.to_owned()));
    }

    Ok(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_signed_in_accounts() {
        let json = br#"{"hosts":{"github.com":[{"state":"success","active":true,"host":"github.com","login":"octocat","tokenSource":"keyring","scopes":"gist, read:org, repo, workflow","gitProtocol":"https"}],"acme.ghe.com":[{"state":"error","active":true,"host":"acme.ghe.com","login":"dev_acme","tokenSource":"keyring","error":"Failed to log in to acme.ghe.com"}],"work.ghe.com":[{"state":"success","active":true,"host":"work.ghe.com","login":"dev_work","tokenSource":"keyring","scopes":"repo","gitProtocol":"https"}]}}"#;

        let accounts = parse(json).unwrap();

        assert_eq!(
            accounts
                .iter()
                .map(|account| (account.host.as_str(), account.login.as_str()))
                .collect::<Vec<_>>(),
            [("github.com", "octocat"), ("work.ghe.com", "dev_work")]
        );
    }

    #[test]
    fn reads_no_accounts() {
        assert_eq!(parse(br#"{"hosts":{}}"#).unwrap(), []);
    }

    #[test]
    fn normalises_hosts() {
        assert_eq!(host(" https://Acme.ghe.com/ ").unwrap(), "acme.ghe.com");
        assert_eq!(host("github.com").unwrap(), "github.com");
        assert!(host("").is_err());
        assert!(host("acme").is_err());
        assert!(host("acme.ghe.com/org").is_err());
        assert!(host("acme.ghe.com --web").is_err());
    }
}
