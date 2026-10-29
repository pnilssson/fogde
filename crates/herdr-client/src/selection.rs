use serde::{Deserialize, Serialize};

use crate::{Client, Error};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Point {
    pub row: u32,
    #[serde(rename = "col")]
    pub column: u16,
}

impl Client {
    pub async fn selection(&self, pane: &str, start: Point, end: Point) -> Result<String, Error> {
        let selected: Selected = self
            .request(
                "pane.selection.read",
                Selection {
                    pane_id: pane,
                    anchor: start,
                    cursor: end,
                },
            )
            .await?;

        Ok(selected.text)
    }
}

#[derive(Serialize)]
struct Selection<'a> {
    pane_id: &'a str,
    anchor: Point,
    cursor: Point,
}

#[derive(Deserialize)]
struct Selected {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::serve_each;

    const SELECTED: &str = r#"{"id":"fogde","result":{"type":"pane_selection","pane_id":"w1:p1","text":"10\nline 11\nline"}}"#;

    #[tokio::test]
    async fn reads_text_between_two_points_in_the_scrollback() {
        let socket = serve_each("selection", vec![SELECTED.to_owned()]);

        let text = Client::new(&socket)
            .selection(
                "w1:p1",
                Point { row: 10, column: 5 },
                Point { row: 12, column: 3 },
            )
            .await
            .unwrap();

        assert_eq!(text, "10\nline 11\nline");
    }

    #[test]
    fn writes_points_as_herdr_names_them() {
        let selection = serde_json::to_value(Selection {
            pane_id: "w1:p1",
            anchor: Point { row: 12, column: 3 },
            cursor: Point { row: 10, column: 5 },
        })
        .unwrap();

        assert_eq!(
            selection,
            serde_json::json!({
                "pane_id": "w1:p1",
                "anchor": { "row": 12, "col": 3 },
                "cursor": { "row": 10, "col": 5 },
            })
        );
    }
}
