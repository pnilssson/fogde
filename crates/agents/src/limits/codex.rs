use serde::Deserialize;

use super::{Answer, Limit, ask};
use crate::{Agent, Error};

const READ: u64 = 1;

#[derive(Deserialize)]
struct Reply {
    id: Option<u64>,
    result: Option<Read>,
    error: Option<Fault>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Read {
    rate_limits: Snapshot,
}

#[derive(Deserialize)]
struct Snapshot {
    primary: Option<Bucket>,
    secondary: Option<Bucket>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bucket {
    used_percent: f64,
    window_duration_mins: Option<i64>,
    resets_at: Option<i64>,
}

#[derive(Deserialize)]
struct Fault {
    message: String,
}

pub async fn codex() -> Result<Vec<Limit>, Error> {
    let requests = format!(
        "{}\n{}\n{}\n",
        serde_json::json!({
            "id": 0,
            "method": "initialize",
            "params": { "clientInfo": { "name": "fogde", "version": env!("CARGO_PKG_VERSION") } },
        }),
        serde_json::json!({ "method": "initialized" }),
        serde_json::json!({ "id": READ, "method": "account/rateLimits/read" }),
    );

    ask(Agent::Codex, &["app-server"], &requests, codex_reply).await
}

fn codex_reply(line: &str) -> Answer {
    let reply: Reply = serde_json::from_str(line).ok()?;
    if reply.id != Some(READ) {
        return None;
    }
    if let Some(fault) = reply.error {
        return Some(Err(fault.message));
    }

    let snapshot = reply.result?.rate_limits;
    Some(Ok([snapshot.primary, snapshot.secondary]
        .into_iter()
        .flatten()
        .filter_map(|window| {
            Some(Limit {
                minutes: window.window_duration_mins?,
                used: window.used_percent,
                resets: window.resets_at?,
            })
        })
        .collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_codex_limits() {
        let line = r#"{"id":1,"result":{"ordinaryUsageAllowed":true,"rateLimits":{"limitId":"codex","limitName":null,"normalModelSlug":null,"primary":{"usedPercent":64,"windowDurationMins":300,"resetsAt":1792710177},"secondary":{"usedPercent":10,"windowDurationMins":10080,"resetsAt":1793296977},"credits":{"hasCredits":true,"unlimited":false,"balance":"7481.0"},"individualLimit":null,"spendControlReached":false,"planType":"plus","rateLimitReachedType":null},"rateLimitsByLimitId":null,"rateLimitResetCredits":{"availableCount":0,"credits":[]},"accountId":"00000000-0000-0000-0000-000000000000","rateLimitUpsell":null}}"#;

        assert_eq!(
            codex_reply(line),
            Some(Ok(vec![
                Limit {
                    minutes: 300,
                    used: 64.0,
                    resets: 1_792_710_177,
                },
                Limit {
                    minutes: 10080,
                    used: 10.0,
                    resets: 1_793_296_977,
                },
            ]))
        );
    }

    #[test]
    fn skips_other_messages() {
        let initialized = r#"{"id":0,"result":{"userAgent":"fogde/0.163.0","codexHome":"/home/dev/.codex","platformFamily":"unix","platformOs":"linux"}}"#;
        let status = r#"{"method":"remoteControl/status/changed","params":{"status":"disabled"}}"#;

        assert_eq!(codex_reply(initialized), None);
        assert_eq!(codex_reply(status), None);
        assert_eq!(codex_reply("not json"), None);
    }

    #[test]
    fn reads_codex_errors() {
        let line = r#"{"error":{"code":-32600,"message":"codex account authentication required to read rate limits"},"id":1}"#;

        assert_eq!(
            codex_reply(line),
            Some(Err(
                "codex account authentication required to read rate limits".to_owned()
            ))
        );
    }
}
