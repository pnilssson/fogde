use serde_json::{Value, json};

use crate::Display;
use crate::fake::{connected, greeted};
use crate::wire;

#[tokio::test]
async fn tells_herdr_the_colors_it_answers_programs_with() {
    let (connection, mut core) = connected().await;
    let light = Display {
        fg: "#18181b".into(),
        bg: "#fafafa".into(),
        cursor: "#18181b".into(),
    };

    let serving = async {
        let request = core.request().await;
        core.reply(
            &request["id"],
            json!({"channel": 1, "terminal": {"address": "t3"}}),
        )
        .await;
        request["params"]["display"].clone()
    };
    let (attached, display) = tokio::join!(connection.attach("t3", 80, 10, light), serving);
    let stream = attached.unwrap();

    assert_eq!(
        display,
        json!({"fg": "#18181b", "bg": "#fafafa", "cursor": "#18181b"})
    );

    let dark = Display {
        fg: "#fafafa".into(),
        bg: "#171717".into(),
        cursor: "#fafafa".into(),
    };
    stream.display(&dark).unwrap();
    let frame = core.frame().await;
    assert_eq!((frame.channel, frame.kind), (1, wire::FACTS));
    assert_eq!(
        serde_json::from_slice::<Value>(&frame.payload).unwrap(),
        json!({"fg": "#fafafa", "bg": "#171717", "cursor": "#fafafa"})
    );
}

const TAGGING: &str = r#"{"id":0,"jsonrpc":"2.0","result":{"core":{"id":"45255384ce5fc955fddc39260e47956f","name":"dev-pc"},"features":["framed","attach","events","status","shell","resize_notifications","automatic_input"],"framed":true,"generation":1,"methods":["hello","terminal.attach"],"version":"0.1.18"}}"#;

async fn attached(hello: &str, confirmed: bool) -> (crate::Stream, crate::fake::Core) {
    let (connection, mut core) = greeted(hello).await;
    let serving = async {
        let request = core.request().await;
        core.reply(
            &request["id"],
            json!({"channel": 1, "automatic_input": confirmed, "terminal": {"address": "t3"}}),
        )
        .await;
    };
    let (attached, ()) = tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);

    (attached.unwrap(), core)
}

#[tokio::test]
async fn tags_terminal_replies_only_when_herdr_offers_and_confirms_it() {
    for (hello, confirmed, kind) in [
        (TAGGING, true, wire::AUTOMATIC),
        (TAGGING, false, wire::DATA),
        (crate::fake::HELLO, true, wire::DATA),
    ] {
        let (stream, mut core) = attached(hello, confirmed).await;

        stream.reply(b"\x1b[?62;22c").unwrap();
        stream.input(b"ls\r").unwrap();

        let reply = core.frame().await;
        assert_eq!(
            (reply.kind, reply.payload),
            (kind, b"\x1b[?62;22c".to_vec())
        );
        assert_eq!(core.frame().await.kind, wire::DATA);
    }
}

#[tokio::test]
async fn tells_herdr_when_the_terminal_gains_and_loses_focus() {
    let (stream, mut core) = attached(TAGGING, true).await;

    stream.focus(true).unwrap();
    stream.focus(false).unwrap();

    for payload in [[1], [0]] {
        let frame = core.frame().await;
        assert_eq!(
            (frame.channel, frame.kind, frame.payload),
            (1, wire::FOCUS, payload.to_vec())
        );
    }
}
