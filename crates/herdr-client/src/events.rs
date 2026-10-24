use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use crate::client::{Connection, reply};
use crate::{Client, Error, Scrollback};

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
    #[serde(rename = "pane.scroll_changed")]
    ScrollChanged { pane_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Scrolled {
        pane: String,
        scrollback: Scrollback,
    },
    Other,
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
        self.event().await.map(drop)
    }

    pub async fn event(&mut self) -> Result<Event, Error> {
        read(&self.0.line().await?)
    }
}

fn read(line: &str) -> Result<Event, Error> {
    match serde_json::from_str(line)? {
        Line::Scrolled { data, .. } => Ok(Event::Scrolled {
            pane: data.pane_id,
            scrollback: data.scroll,
        }),
        Line::Event { .. } => Ok(Event::Other),
        Line::Reply(_) => reply::<IgnoredAny>(line).map(|_| Event::Other),
    }
}

#[derive(Serialize)]
struct Subscribe<'a> {
    subscriptions: &'a [Subscription],
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Line {
    Scrolled {
        #[serde(rename = "event")]
        _event: ScrollChanged,
        data: ScrollData,
    },
    Event {
        #[serde(rename = "event")]
        _event: String,
    },
    Reply(IgnoredAny),
}

#[derive(Deserialize)]
enum ScrollChanged {
    #[serde(rename = "pane.scroll_changed")]
    Changed,
}

#[derive(Deserialize)]
struct ScrollData {
    pane_id: String,
    scroll: Scrollback,
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
            Subscription::ScrollChanged {
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
                {"type": "pane.scroll_changed", "pane_id": "w1:p1"},
            ]})
        );
    }

    #[test]
    fn reads_events() {
        let status = r#"{"data":{"agent":"claude","agent_status":"blocked","pane_id":"w1:p1","workspace_id":"w1"},"event":"pane.agent_status_changed"}"#;
        let renamed = r#"{"data":{"label":"renamed","type":"workspace_renamed","workspace_id":"w1"},"event":"workspace_renamed"}"#;

        assert_eq!(read(status).unwrap(), Event::Other);
        assert_eq!(read(renamed).unwrap(), Event::Other);
    }

    #[test]
    fn reads_scroll_changes() {
        let scrolled = r#"{"data":{"pane_id":"w1:p1","scroll":{"max_offset_from_bottom":240,"offset_from_bottom":12,"viewport_rows":30},"workspace_id":"w1"},"event":"pane.scroll_changed"}"#;

        assert_eq!(
            read(scrolled).unwrap(),
            Event::Scrolled {
                pane: "w1:p1".into(),
                scrollback: Scrollback {
                    offset: 12,
                    max: 240,
                    rows: 30,
                },
            }
        );
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
