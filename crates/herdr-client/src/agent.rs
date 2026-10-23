use std::time::Duration;

use serde::Serialize;
use serde::de::IgnoredAny;
use tokio::time::{Instant, sleep};

use crate::{Client, Error};

const BUSY: &str = "agent_pane_busy";
const SHELL_WAIT: Duration = Duration::from_secs(10);
const RETRY: Duration = Duration::from_millis(50);

impl Client {
    pub async fn start_agent(
        &self,
        name: &str,
        kind: &str,
        pane: &str,
        args: &[String],
    ) -> Result<(), Error> {
        let deadline = Instant::now() + SHELL_WAIT;
        loop {
            let request = Start {
                name,
                kind,
                pane_id: pane,
                args,
            };
            match self.request::<IgnoredAny>("agent.start", request).await {
                Err(Error::Herdr { code, .. }) if code == BUSY && Instant::now() < deadline => {
                    sleep(RETRY).await;
                }
                result => return result.map(|_| ()),
            }
        }
    }
}

#[derive(Serialize)]
struct Start<'a> {
    name: &'a str,
    kind: &'a str,
    pane_id: &'a str,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    args: &'a [String],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::serve_each;

    const STARTED: &str = r#"{"agent":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"06041bb7-9bd5-4729-bc8b-a3fdda2089b8"},"agent_status":"idle","cwd":"C:\\Users\\dev\\app","focused":false,"interactive_ready":true,"name":"t4","pane_id":"w4:p1","revision":2,"state_change_seq":2,"tab_id":"w4:t1","terminal_id":"term_65e20654599884","terminal_title":"✳ Claude Code","terminal_title_stripped":"Claude Code","workspace_id":"w4"},"argv":["claude"],"type":"agent_started"}"#;

    #[tokio::test]
    async fn starts_an_agent_once_the_shell_is_ready() {
        let busy = r#"{"id":"fogde","error":{"code":"agent_pane_busy","message":"agent target pane w4:p1 is not an available shell"}}"#;
        let socket = serve_each(
            "busy",
            vec![
                busy.to_owned(),
                busy.to_owned(),
                format!(r#"{{"id":"fogde","result":{STARTED}}}"#),
            ],
        );

        Client::new(&socket)
            .start_agent("codex-w4", "codex", "w4:p1", &[])
            .await
            .unwrap();
    }
}
