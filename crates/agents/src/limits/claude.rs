use serde::Deserialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::{Answer, Limit, ask};
use crate::{Agent, Error};

const USAGE: &str = "usage";
const CLAUDE_ARGS: [&str; 11] = [
    "-p",
    "--input-format",
    "stream-json",
    "--output-format",
    "stream-json",
    "--verbose",
    "--no-session-persistence",
    "--setting-sources",
    "",
    "--strict-mcp-config",
    "--disable-slash-commands",
];

#[derive(Deserialize)]
struct Control {
    response: Verdict,
}

#[derive(Deserialize)]
struct Verdict {
    request_id: String,
    response: Option<Plan>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct Plan {
    rate_limits: Option<Windows>,
}

#[derive(Deserialize)]
struct Windows {
    five_hour: Option<Utilization>,
    seven_day: Option<Utilization>,
}

#[derive(Deserialize)]
struct Utilization {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

pub async fn claude() -> Result<Vec<Limit>, Error> {
    let request = serde_json::json!({
        "type": "control_request",
        "request_id": USAGE,
        "request": { "subtype": "get_usage", "skip_behaviors": true },
    });

    ask(
        Agent::Claude,
        &CLAUDE_ARGS,
        &format!("{request}\n"),
        claude_reply,
    )
    .await
}

fn claude_reply(line: &str) -> Answer {
    let verdict = serde_json::from_str::<Control>(line).ok()?.response;
    if verdict.request_id != USAGE {
        return None;
    }
    if let Some(error) = verdict.error {
        return Some(Err(error));
    }

    let windows = verdict.response?.rate_limits;
    Some(Ok(windows
        .into_iter()
        .flat_map(|windows| [(300, windows.five_hour), (10080, windows.seven_day)])
        .filter_map(|(minutes, window)| {
            let window = window?;

            Some(Limit {
                minutes,
                used: window.utilization?,
                resets: OffsetDateTime::parse(&window.resets_at?, &Rfc3339)
                    .ok()?
                    .unix_timestamp(),
            })
        })
        .collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_claude_limits() {
        let line = r#"{"type":"control_response","response":{"subtype":"success","request_id":"usage","response":{"session":{"total_cost_usd":0,"total_duration_ms":1029,"model_usage":{}},"subscription_type":"pro","rate_limits_available":true,"rate_limits":{"five_hour":{"utilization":42,"resets_at":"2026-10-24T18:20:00.442556+00:00","limit_dollars":null,"locked_reason":null},"seven_day":{"utilization":95,"resets_at":"2026-10-27T16:00:00.442574+00:00"},"seven_day_opus":null}}}}"#;

        assert_eq!(
            claude_reply(line),
            Some(Ok(vec![
                Limit {
                    minutes: 300,
                    used: 42.0,
                    resets: 1_792_866_000,
                },
                Limit {
                    minutes: 10080,
                    used: 95.0,
                    resets: 1_793_116_800,
                },
            ]))
        );
    }

    #[test]
    fn skips_claude_plans_without_limits() {
        let api_key = r#"{"type":"control_response","response":{"subtype":"success","request_id":"usage","response":{"subscription_type":null,"rate_limits_available":false,"rate_limits":null}}}"#;
        let other = r#"{"type":"control_response","response":{"subtype":"success","request_id":"other","response":{}}}"#;
        let system = r#"{"type":"system","subtype":"hook_started","hook_id":"1"}"#;

        assert_eq!(claude_reply(api_key), Some(Ok(vec![])));
        assert_eq!(claude_reply(other), None);
        assert_eq!(claude_reply(system), None);
    }

    #[test]
    fn reads_claude_errors() {
        let line = r#"{"type":"control_response","response":{"subtype":"error","request_id":"usage","error":"Unsupported control request subtype: get_usage"}}"#;

        assert_eq!(
            claude_reply(line),
            Some(Err(
                "Unsupported control request subtype: get_usage".to_owned()
            ))
        );
    }
}
