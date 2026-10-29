use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use crate::pane::Target;
use crate::{Client, Error};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct Scrollback {
    #[serde(rename = "offset_from_bottom")]
    pub offset: u64,
    #[serde(rename = "max_offset_from_bottom")]
    pub max: u64,
    #[serde(rename = "viewport_rows")]
    pub rows: u64,
}

impl Client {
    pub async fn scrollback(&self, pane: &str) -> Result<Option<Scrollback>, Error> {
        let info: Info = self.request("pane.get", Target { pane_id: pane }).await?;

        Ok(info.pane.scroll)
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
}

#[derive(Deserialize)]
struct Info {
    pane: Pane,
}

#[derive(Deserialize)]
struct Pane {
    scroll: Option<Scrollback>,
}

#[derive(Serialize)]
struct Scroll<'a> {
    pane_id: &'a str,
    offset_from_bottom: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::serve_each;

    const PANE: &str = r#"{"id":"fogde","result":{"type":"pane_info","pane":{"pane_id":"w1:p1","terminal_id":"term_65f00b8f910ea1","workspace_id":"w1","tab_id":"w1:t1","focused":true,"cwd":"C:\\repos\\app\\","terminal_title":"C:\\WINDOWS\\system32\\cmd.exe - echo  line 5994","terminal_title_stripped":"C:\\WINDOWS\\system32\\cmd.exe - echo  line 5994","agent_status":"unknown","scroll":{"offset_from_bottom":0,"max_offset_from_bottom":507,"viewport_rows":40},"revision":166}}}"#;

    #[tokio::test]
    async fn reads_where_one_pane_is_scrolled() {
        let socket = serve_each("scrollback", vec![PANE.to_owned()]);

        let scrollback = Client::new(&socket).scrollback("w1:p1").await.unwrap();

        assert_eq!(
            scrollback,
            Some(Scrollback {
                offset: 0,
                max: 507,
                rows: 40,
            })
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
}
