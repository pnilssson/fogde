use std::path::PathBuf;

use serde::de::IgnoredAny;
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
    #[serde(rename = "terminal_title")]
    pub title: Option<String>,
    #[serde(rename = "terminal_title_stripped")]
    pub stripped: Option<String>,
    #[serde(rename = "scroll")]
    pub scrollback: Option<Scrollback>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct Scrollback {
    #[serde(rename = "offset_from_bottom")]
    pub offset: u64,
    #[serde(rename = "max_offset_from_bottom")]
    pub max: u64,
    #[serde(rename = "viewport_rows")]
    pub rows: u64,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum Key {
    #[serde(rename = "shift+enter")]
    ShiftEnter,
    #[serde(rename = "ctrl+enter")]
    CtrlEnter,
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

    pub async fn focus_pane(&self, pane: &str) -> Result<(), Error> {
        self.request::<IgnoredAny>("pane.focus", Focus { pane_id: pane })
            .await?;

        Ok(())
    }

    pub async fn scroll_pane(&self, pane: &str, offset: u64) -> Result<(), Error> {
        self.request::<IgnoredAny>(
            "pane.scroll",
            Scroll {
                pane_id: pane,
                offset_from_bottom: offset,
            },
        )
        .await?;

        Ok(())
    }

    pub async fn send_keys(&self, pane: &str, keys: &[Key]) -> Result<(), Error> {
        self.request::<IgnoredAny>(
            "pane.send_keys",
            Keys {
                pane_id: pane,
                keys,
            },
        )
        .await?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct PaneList {
    panes: Vec<Pane>,
}

#[derive(Serialize)]
struct Focus<'a> {
    pane_id: &'a str,
}

#[derive(Serialize)]
struct Scroll<'a> {
    pane_id: &'a str,
    offset_from_bottom: u64,
}

#[derive(Serialize)]
struct Keys<'a> {
    pane_id: &'a str,
    keys: &'a [Key],
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = r#"{"panes":[{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"0f6e1c52-3a7b-4d2e-9c1a-5b8e2f4d7a10"},"agent_status":"idle","cwd":"C:\\Users\\dev\\app","focused":true,"pane_id":"w3:p1","revision":10,"scroll":{"max_offset_from_bottom":0,"offset_from_bottom":0,"viewport_rows":56},"tab_id":"w3:t1","terminal_id":"term_65e188ff9e68f3","terminal_title":"✳ Claude Code","terminal_title_stripped":"Claude Code","workspace_id":"w3"},{"agent_status":"unknown","cwd":"C:\\Users\\dev\\notes","focused":false,"pane_id":"w3:p2","revision":0,"scroll":{"max_offset_from_bottom":20,"offset_from_bottom":0,"viewport_rows":14},"tab_id":"w3:t1","terminal_id":"term_65e188ff9f8c04","workspace_id":"w3"}],"type":"pane_list"}"#;

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
                    cwd: Some(PathBuf::from(r"C:\Users\dev\app")),
                    title: Some("✳ Claude Code".into()),
                    stripped: Some("Claude Code".into()),
                    scrollback: Some(Scrollback {
                        offset: 0,
                        max: 0,
                        rows: 56,
                    }),
                },
                Pane {
                    id: "w3:p2".into(),
                    workspace: "w3".into(),
                    agent: None,
                    status: Status::Unknown,
                    session: None,
                    cwd: Some(PathBuf::from(r"C:\Users\dev\notes")),
                    title: None,
                    stripped: None,
                    scrollback: Some(Scrollback {
                        offset: 0,
                        max: 20,
                        rows: 14,
                    }),
                },
            ]
        );
    }

    #[test]
    fn writes_scroll() {
        let scroll = serde_json::to_value(Scroll {
            pane_id: "w3:p1",
            offset_from_bottom: 12,
        })
        .unwrap();

        assert_eq!(
            scroll,
            serde_json::json!({"pane_id": "w3:p1", "offset_from_bottom": 12})
        );
    }

    #[test]
    fn writes_send_keys() {
        let keys = serde_json::to_value(Keys {
            pane_id: "w3:p1",
            keys: &[Key::ShiftEnter, Key::CtrlEnter],
        })
        .unwrap();

        assert_eq!(
            keys,
            serde_json::json!({"pane_id": "w3:p1", "keys": ["shift+enter", "ctrl+enter"]})
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
