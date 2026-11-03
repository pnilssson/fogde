use std::collections::BTreeMap;
use std::path::Path;

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use crate::{Connection, Error};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Terminal {
    pub address: String,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub turn: u64,
    #[serde(default)]
    pub foreground: Option<Foreground>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub state: String,
    #[serde(default)]
    pub app: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Foreground {
    pub pid: i32,
    pub name: String,
    #[serde(default)]
    pub cwd: String,
}

pub struct Run<'a> {
    pub argv: &'a [String],
    pub cwd: &'a Path,
    pub env: &'a BTreeMap<String, String>,
    pub labels: &'a BTreeMap<String, String>,
    pub size: Option<Size>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

impl Connection {
    pub async fn run(&self, run: Run<'_>) -> Result<Terminal, Error> {
        self.call(
            "terminal.run",
            Started {
                argv: run.argv,
                cwd: run.cwd,
                env: run.env,
                labels: run.labels,
                rm: true,
                keep: true,
                size: run.size,
            },
        )
        .await
    }

    pub async fn terminals(&self) -> Result<Vec<Terminal>, Error> {
        let list: List = self.call("terminal.list", Empty {}).await?;

        Ok(list.terminals)
    }

    pub async fn label(
        &self,
        terminal: &str,
        labels: &BTreeMap<String, String>,
    ) -> Result<(), Error> {
        self.call::<IgnoredAny>("terminal.label", Labelled { terminal, labels })
            .await
            .map(drop)
    }

    pub async fn kill(&self, terminal: &str) -> Result<(), Error> {
        match self
            .call::<IgnoredAny>("terminal.kill", Target { terminal })
            .await
        {
            Err(error) if error.is_gone() => Ok(()),
            result => result.map(drop),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Empty {}

#[derive(Serialize)]
pub(crate) struct Target<'a> {
    pub(crate) terminal: &'a str,
}

#[derive(Serialize)]
struct Labelled<'a> {
    terminal: &'a str,
    labels: &'a BTreeMap<String, String>,
}

#[derive(Serialize)]
struct Started<'a> {
    argv: &'a [String],
    cwd: &'a Path,
    env: &'a BTreeMap<String, String>,
    labels: &'a BTreeMap<String, String>,
    rm: bool,
    keep: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<Size>,
}

#[derive(Deserialize)]
struct List {
    terminals: Vec<Terminal>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const LIST: &str = r#"{"hidden_finished":0,"terminals":[{"address":"t2","agent":"claude-code","argv":["C:\\Program Files\\PowerShell\\7\\pwsh.exe"],"cols":120,"created_ms":1793549649674,"cwd":"C:\\Users\\dev\\app","die_with_parent":false,"exit_code":null,"finished_ms":null,"foreground":{"argv":["C:\\Users\\dev\\.local\\bin\\claude.EXE"],"cwd":"C:\\Users\\dev\\app","name":"claude.EXE","pid":22364},"identity":"ghostty","keep":false,"labels":{"fogde":"Claude"},"name":null,"notify":true,"parent":null,"pid":17004,"reason":null,"reported":true,"rm":true,"rows":40,"screen":"exact","signal":null,"state":"live","status":{"app":"claude-code","kind":"permission","msg":"approve Bash: mkdir newdir","state":"blocked"},"status_ms":1793549649933,"template":null,"title":"✳ Make a folder","turn":2},{"address":"t3","argv":["/bin/zsh"],"cols":80,"created_ms":1793549650000,"cwd":"/repos/app","labels":{},"pid":4242,"rows":24,"screen":"exact","state":"live","status":null,"title":"","turn":0}]}"#;

    #[test]
    fn reads_an_agent_and_a_shell() {
        let list: List = serde_json::from_str(LIST).unwrap();

        assert_eq!(
            list.terminals,
            [
                Terminal {
                    address: "t2".into(),
                    labels: BTreeMap::from([("fogde".into(), "Claude".into())]),
                    cwd: r"C:\Users\dev\app".into(),
                    pid: Some(17004),
                    title: "✳ Make a folder".into(),
                    status: Some(Status {
                        state: "blocked".into(),
                        app: Some("claude-code".into()),
                    }),
                    turn: 2,
                    foreground: Some(Foreground {
                        pid: 22364,
                        name: "claude.EXE".into(),
                        cwd: r"C:\Users\dev\app".into(),
                    }),
                },
                Terminal {
                    address: "t3".into(),
                    labels: BTreeMap::new(),
                    cwd: "/repos/app".into(),
                    pid: Some(4242),
                    title: String::new(),
                    status: None,
                    turn: 0,
                    foreground: None,
                },
            ]
        );
    }

    #[test]
    fn starts_a_kept_terminal_at_its_size_and_removes_it_once_it_exits() {
        let env = BTreeMap::from([("XDG_CONFIG_HOME".to_owned(), String::new())]);
        let labels = BTreeMap::from([("fogde".to_owned(), "Terminal".to_owned())]);
        let argv = ["/bin/zsh".to_owned()];
        let started = Started {
            argv: &argv,
            cwd: Path::new("/repos/app"),
            env: &env,
            labels: &labels,
            rm: true,
            keep: true,
            size: Some(Size {
                cols: 120,
                rows: 40,
            }),
        };

        assert_eq!(
            serde_json::to_value(started).unwrap(),
            json!({
                "argv": ["/bin/zsh"],
                "cwd": "/repos/app",
                "env": {"XDG_CONFIG_HOME": ""},
                "labels": {"fogde": "Terminal"},
                "rm": true,
                "keep": true,
                "size": {"cols": 120, "rows": 40},
            })
        );
    }
}
