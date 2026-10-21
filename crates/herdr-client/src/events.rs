use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use crate::client::{Connection, reply};
use crate::{Client, Error};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum Subscription {
    #[serde(rename = "workspace.created")]
    WorkspaceCreated,
    #[serde(rename = "workspace.closed")]
    WorkspaceClosed,
    #[serde(rename = "workspace.renamed")]
    WorkspaceRenamed,
    #[serde(rename = "pane.created")]
    PaneCreated,
    #[serde(rename = "pane.closed")]
    PaneClosed,
    #[serde(rename = "pane.exited")]
    PaneExited,
    #[serde(rename = "pane.updated")]
    PaneUpdated,
    #[serde(rename = "pane.agent_detected")]
    AgentDetected,
    #[serde(rename = "pane.agent_status_changed")]
    AgentStatus { pane_id: String },
}

pub struct Events(Connection);

impl Client {
    pub async fn subscribe(&self, subscriptions: &[Subscription]) -> Result<Events, Error> {
        let mut connection = self
            .open("events.subscribe", Subscribe { subscriptions })
            .await?;
        reply::<IgnoredAny>(&connection.line().await?)?;

        Ok(Events(connection))
    }
}

impl Events {
    pub async fn next(&mut self) -> Result<(), Error> {
        read(&self.0.line().await?)
    }
}

fn read(line: &str) -> Result<(), Error> {
    match serde_json::from_str(line)? {
        Line::Event { .. } => Ok(()),
        Line::Reply(_) => reply::<IgnoredAny>(line).map(drop),
    }
}

#[derive(Serialize)]
struct Subscribe<'a> {
    subscriptions: &'a [Subscription],
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Line {
    Event {
        #[serde(rename = "event")]
        _event: String,
    },
    Reply(IgnoredAny),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn writes_subscriptions() {
        let subscriptions = [
            Subscription::PaneCreated,
            Subscription::AgentStatus {
                pane_id: "w1:p1".into(),
            },
        ];

        assert_eq!(
            serde_json::to_value(Subscribe {
                subscriptions: &subscriptions
            })
            .unwrap(),
            json!({"subscriptions": [
                {"type": "pane.created"},
                {"type": "pane.agent_status_changed", "pane_id": "w1:p1"},
            ]})
        );
    }

    #[test]
    fn reads_events() {
        let status = r#"{"data":{"agent":"claude","agent_status":"blocked","pane_id":"w1:p1","workspace_id":"w1"},"event":"pane.agent_status_changed"}"#;
        let renamed = r#"{"data":{"label":"renamed","type":"workspace_renamed","workspace_id":"w1"},"event":"workspace_renamed"}"#;

        assert!(read(status).is_ok());
        assert!(read(renamed).is_ok());
    }

    #[test]
    fn reports_lost_events() {
        let lost =
            r#"{"id":"fogde","error":{"code":"events_lost","message":"subscriber fell behind"}}"#;

        assert!(matches!(
            read(lost),
            Err(Error::Herdr { code, .. }) if code == "events_lost"
        ));
    }
}
