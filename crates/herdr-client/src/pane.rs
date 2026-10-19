use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::client::Empty;
use crate::{Client, Error};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Pane {
    #[serde(rename = "pane_id")]
    pub id: String,
    #[serde(rename = "workspace_id")]
    pub workspace: String,
    pub agent: Option<String>,
    #[serde(rename = "agent_status")]
    pub status: Status,
    #[serde(rename = "agent_session")]
    pub session: Option<Session>,
    pub cwd: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idle,
    Working,
    Blocked,
    Done,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Session {
    Id(String),
    Path(PathBuf),
}

impl Client {
    pub async fn panes(&self) -> Result<Vec<Pane>, Error> {
        let list: PaneList = self.request("pane.list", Empty {}).await?;

        Ok(list.panes)
    }

    pub async fn start_agent(
        &self,
        name: &str,
        kind: &str,
        pane: &str,
        args: &[&str],
    ) -> Result<Pane, Error> {
        let started: Started = self
            .request(
                "agent.start",
                Start {
                    name,
                    kind,
                    pane_id: pane,
                    args,
                },
            )
            .await?;

        Ok(started.agent)
    }
}

#[derive(Deserialize)]
struct PaneList {
    panes: Vec<Pane>,
}

#[derive(Serialize)]
struct Start<'a> {
    name: &'a str,
    kind: &'a str,
    pane_id: &'a str,
    args: &'a [&'a str],
}

#[derive(Deserialize)]
struct Started {
    agent: Pane,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = r#"{"panes":[{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"0f6e1c52-3a7b-4d2e-9c1a-5b8e2f4d7a10"},"agent_status":"idle","cwd":"C:\\Users\\dev\\fogde","focused":true,"pane_id":"w3:p1","revision":10,"scroll":{"max_offset_from_bottom":0,"offset_from_bottom":0,"viewport_rows":56},"tab_id":"w3:t1","terminal_id":"term_65e188ff9e68f3","terminal_title":"✳ Claude Code","terminal_title_stripped":"Claude Code","workspace_id":"w3"},{"agent_status":"unknown","cwd":"C:\\Users\\dev\\notes","focused":false,"pane_id":"w3:p2","revision":0,"scroll":{"max_offset_from_bottom":20,"offset_from_bottom":0,"viewport_rows":14},"tab_id":"w3:t1","terminal_id":"term_65e188ff9f8c04","workspace_id":"w3"}],"type":"pane_list"}"#;

    const STARTED: &str = r#"{"agent":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"06041bb7-9bd5-4729-bc8b-a3fdda2089b8"},"agent_status":"idle","cwd":"C:\\Users\\dev\\fogde","focused":false,"interactive_ready":true,"name":"t4","pane_id":"w4:p1","revision":2,"state_change_seq":2,"tab_id":"w4:t1","terminal_id":"term_65e20654599884","terminal_title":"✳ Claude Code","terminal_title_stripped":"Claude Code","workspace_id":"w4"},"argv":["claude"],"type":"agent_started"}"#;

    #[test]
    fn reads_agent_and_shell_panes() {
        let list: PaneList = serde_json::from_str(LIST).unwrap();

        assert_eq!(
            list.panes,
            vec![
                Pane {
                    id: "w3:p1".into(),
                    workspace: "w3".into(),
                    agent: Some("claude".into()),
                    status: Status::Idle,
                    session: Some(Session::Id("0f6e1c52-3a7b-4d2e-9c1a-5b8e2f4d7a10".into())),
                    cwd: Some(PathBuf::from(r"C:\Users\dev\fogde")),
                },
                Pane {
                    id: "w3:p2".into(),
                    workspace: "w3".into(),
                    agent: None,
                    status: Status::Unknown,
                    session: None,
                    cwd: Some(PathBuf::from(r"C:\Users\dev\notes")),
                },
            ]
        );
    }

    #[test]
    fn reads_started_agent() {
        let started: Started = serde_json::from_str(STARTED).unwrap();

        assert_eq!(
            (started.agent.id.as_str(), started.agent.status),
            ("w4:p1", Status::Idle)
        );
    }

    #[test]
    fn reads_session_path() {
        let session: Session = serde_json::from_str(
            r#"{"agent":"pi","kind":"path","source":"herdr:pi","value":"/tmp/s.jsonl"}"#,
        )
        .unwrap();

        assert_eq!(session, Session::Path(PathBuf::from("/tmp/s.jsonl")));
    }

    #[test]
    fn treats_new_status_as_unknown() {
        let status: Status = serde_json::from_str(r#""paused""#).unwrap();

        assert_eq!(status, Status::Unknown);
    }
}
