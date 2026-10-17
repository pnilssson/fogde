use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Client, Error};

impl Client {
    pub async fn agents(&mut self) -> Result<Vec<Agent>, Error> {
        let list: AgentList = self.request("agent.list", Empty {}).await?;

        Ok(list.agents)
    }
}

#[derive(Serialize)]
struct Empty {}

#[derive(Deserialize)]
struct AgentList {
    agents: Vec<Agent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Agent {
    pub pane_id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = r#"{"agents":[{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"0f6e1c52-3a7b-4d2e-9c1a-5b8e2f4d7a10"},"agent_status":"idle","cwd":"C:\\Users\\dev\\fogde","focused":false,"interactive_ready":true,"name":"demo","pane_id":"w3:p1","revision":2,"state_change_seq":4,"tab_id":"w3:t1","terminal_id":"term_65e07f81043053","terminal_title":"✳ Claude Code","terminal_title_stripped":"Claude Code","workspace_id":"w3"}],"type":"agent_list"}"#;

    #[test]
    fn reads_agent_list() {
        let list: AgentList = serde_json::from_str(LIST).unwrap();

        assert_eq!(
            list.agents,
            vec![Agent {
                pane_id: "w3:p1".into(),
                agent: Some("claude".into()),
                status: Status::Idle,
                session: Some(Session::Id("0f6e1c52-3a7b-4d2e-9c1a-5b8e2f4d7a10".into())),
                cwd: Some(PathBuf::from(r"C:\Users\dev\fogde")),
            }]
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
