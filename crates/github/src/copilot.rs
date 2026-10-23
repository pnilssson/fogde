use std::collections::HashMap;

use serde::Deserialize;

use crate::Error;
use crate::gh::{Gh, failure};

const QUOTAS: [&str; 2] = ["premium_interactions", "chat"];
const DAY: i64 = 86_400;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quota {
    pub left: f64,
    pub resets: i64,
}

#[derive(Deserialize)]
struct User {
    quota_reset_date_utc: Option<String>,
    #[serde(default)]
    quota_snapshots: HashMap<String, Snapshot>,
}

#[derive(Deserialize)]
struct Snapshot {
    #[serde(default)]
    entitlement: f64,
    #[serde(default)]
    unlimited: bool,
    percent_remaining: Option<f64>,
}

impl Gh {
    pub async fn copilot_quota(&self) -> Result<Option<Quota>, Error> {
        let output = self.run(&["api", "copilot_internal/user"]).await?;
        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        parse(&output.stdout)
    }
}

fn parse(json: &[u8]) -> Result<Option<Quota>, Error> {
    let user: User = serde_json::from_slice(json)?;
    let Some(resets) = user.quota_reset_date_utc.as_deref().and_then(midnight) else {
        return Ok(None);
    };

    Ok(QUOTAS
        .iter()
        .filter_map(|name| user.quota_snapshots.get(*name))
        .find(|snapshot| !snapshot.unlimited && snapshot.entitlement > 0.0)
        .and_then(|snapshot| snapshot.percent_remaining)
        .map(|left| Quota { left, resets }))
}

fn midnight(date: &str) -> Option<i64> {
    let mut parts = date.get(..10)?.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;

    Some((era * 146_097 + of_cycle - 719_468) * DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FREE: &[u8] = br#"{"access_type_sku":"free_limited_copilot","can_signup_for_limited":false,"chat_enabled":true,"cli_enabled":true,"copilot_plan":"individual","quota_reset_date":"2026-11-01","quota_reset_date_utc":"2026-11-01T00:00:00.000Z","quota_snapshots":{"chat":{"credits_used":0,"entitlement":200,"has_quota":true,"overage_count":0,"overage_permitted":false,"percent_remaining":99.8,"quota_id":"chat","quota_remaining":199.7,"quota_reset_at":0,"remaining":199,"timestamp_utc":"2026-10-22T20:44:15.790Z","token_based_billing":true,"unlimited":false},"completions":{"credits_used":7,"entitlement":2000,"has_quota":true,"overage_count":0,"overage_permitted":false,"percent_remaining":99.6,"quota_id":"completions","quota_remaining":1993,"quota_reset_at":0,"remaining":1993,"timestamp_utc":"2026-10-22T20:44:15.790Z","token_based_billing":true,"unlimited":false},"premium_interactions":{"credits_used":0,"entitlement":0,"has_quota":false,"overage_count":0,"overage_permitted":false,"percent_remaining":0,"quota_id":"premium_interactions","quota_remaining":0,"quota_reset_at":0,"remaining":0,"timestamp_utc":"2026-10-22T20:44:15.790Z","token_based_billing":true,"unlimited":false}},"token_based_billing":true}"#;

    #[test]
    fn reads_the_free_plan_chat_quota() {
        assert_eq!(
            parse(FREE).unwrap(),
            Some(Quota {
                left: 99.8,
                resets: 1_793_491_200,
            })
        );
    }

    #[test]
    fn prefers_premium_requests_on_paid_plans() {
        let paid = String::from_utf8_lossy(FREE).replace(
            r#""entitlement":0,"has_quota":false,"overage_count":0,"overage_permitted":false,"percent_remaining":0"#,
            r#""entitlement":300,"has_quota":true,"overage_count":0,"overage_permitted":false,"percent_remaining":41.5"#,
        );

        assert_eq!(
            parse(paid.as_bytes()).unwrap().map(|quota| quota.left),
            Some(41.5)
        );
    }

    #[test]
    fn skips_unlimited_quotas() {
        let unlimited =
            String::from_utf8_lossy(FREE).replace(r#""unlimited":false"#, r#""unlimited":true"#);

        assert_eq!(parse(unlimited.as_bytes()).unwrap(), None);
    }

    #[test]
    fn converts_reset_dates() {
        assert_eq!(midnight("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(midnight("2000-03-01"), Some(951_868_800));
        assert_eq!(midnight("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        assert_eq!(midnight("soon"), None);
    }
}
