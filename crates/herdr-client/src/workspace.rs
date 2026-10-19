use std::collections::BTreeMap;
use std::path::Path;

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use crate::client::Empty;
use crate::{Client, Error, Pane};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    #[serde(rename = "workspace_id")]
    pub id: String,
    pub label: String,
}

impl Client {
    pub async fn workspaces(&self) -> Result<Vec<Workspace>, Error> {
        let list: WorkspaceList = self.request("workspace.list", Empty {}).await?;

        Ok(list.workspaces)
    }

    pub async fn create_workspace(
        &self,
        cwd: &Path,
        label: &str,
        env: &BTreeMap<String, String>,
    ) -> Result<Pane, Error> {
        let created: Created = self
            .request("workspace.create", Create { cwd, label, env })
            .await?;

        Ok(created.root_pane)
    }

    pub async fn close_workspace(&self, id: &str) -> Result<(), Error> {
        self.request::<IgnoredAny>("workspace.close", Close { workspace_id: id })
            .await?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct WorkspaceList {
    workspaces: Vec<Workspace>,
}

#[derive(Serialize)]
struct Create<'a> {
    cwd: &'a Path,
    label: &'a str,
    env: &'a BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Created {
    root_pane: Pane,
}

#[derive(Serialize)]
struct Close<'a> {
    workspace_id: &'a str,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::Status;

    const CREATED: &str = r#"{"root_pane":{"agent_status":"unknown","cwd":"C:\\Users\\dev\\fogde\\","focused":true,"pane_id":"w1:p1","revision":0,"scroll":{"max_offset_from_bottom":0,"offset_from_bottom":0,"viewport_rows":40},"tab_id":"w1:t1","terminal_id":"term_65e205e7c790d1","workspace_id":"w1"},"tab":{"agent_status":"unknown","focused":true,"label":"1","number":1,"pane_count":1,"tab_id":"w1:t1","workspace_id":"w1"},"type":"workspace_created","workspace":{"active_tab_id":"w1:t1","agent_status":"unknown","focused":true,"label":"t1","number":1,"pane_count":1,"tab_count":1,"workspace_id":"w1"}}"#;

    const LIST: &str = r#"{"type":"workspace_list","workspaces":[{"active_tab_id":"w4:t1","agent_status":"working","focused":true,"label":"api","number":1,"pane_count":3,"tab_count":1,"workspace_id":"w4"},{"active_tab_id":"w5:t1","agent_status":"unknown","focused":false,"label":"notes","number":2,"pane_count":1,"tab_count":1,"workspace_id":"w5"}]}"#;

    #[test]
    fn reads_created_workspace() {
        let created: Created = serde_json::from_str(CREATED).unwrap();

        assert_eq!(
            (
                created.root_pane.id.as_str(),
                created.root_pane.workspace.as_str(),
                created.root_pane.status
            ),
            ("w1:p1", "w1", Status::Unknown)
        );
    }

    #[test]
    fn reads_workspace_list() {
        let list: WorkspaceList = serde_json::from_str(LIST).unwrap();

        assert_eq!(
            list.workspaces,
            vec![
                Workspace {
                    id: "w4".into(),
                    label: "api".into(),
                },
                Workspace {
                    id: "w5".into(),
                    label: "notes".into(),
                },
            ]
        );
    }

    #[test]
    fn gives_panes_the_env() {
        let env = BTreeMap::from([("XDG_CONFIG_HOME".to_owned(), String::new())]);
        let create = Create {
            cwd: Path::new("/repos/fogde"),
            label: "fix-login",
            env: &env,
        };

        assert_eq!(
            serde_json::to_value(create).unwrap(),
            json!({"cwd": "/repos/fogde", "label": "fix-login", "env": {"XDG_CONFIG_HOME": ""}})
        );
    }
}
