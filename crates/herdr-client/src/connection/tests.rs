use serde_json::{Value, json};

use super::*;
use crate::Display;
use crate::fake::connected;
use crate::wire::{CLOSE, Frame};

const GONE: &str = r#"{"error":{"code":-32000,"data":{"hint":"see `hc ls --all`","kind":"not_found"},"message":"t3 was removed"},"id":ID,"jsonrpc":"2.0"}"#;
const PAINT: &[u8] = b"\x18\x1b[!p\x1b[?2026l\x1b[=0;1u\x1b[H\x1b[2JWindows PowerShell\r\n";

#[tokio::test]
async fn reads_the_version_herdr_greets_with() {
    let (connection, _core) = connected().await;

    assert_eq!(connection.version(), "0.1.16");
}

#[tokio::test]
async fn answers_each_call_by_its_id() {
    let (connection, mut core) = connected().await;

    let answering = async {
        let first = core.request().await;
        let second = core.request().await;
        core.reply(&second["id"], json!({"seq": 2})).await;
        core.reply(&first["id"], json!({"seq": 1})).await;
        (first["method"].clone(), second["method"].clone())
    };
    let (first, second, methods) = tokio::join!(
        connection.call::<Value>("terminal.list", json!({})),
        connection.call::<Value>("server.status", json!({})),
        answering,
    );

    assert_eq!(first.unwrap(), json!({"seq": 1}));
    assert_eq!(second.unwrap(), json!({"seq": 2}));
    assert_eq!(methods, (json!("terminal.list"), json!("server.status")));
}

#[tokio::test]
async fn reports_the_kind_of_error_herdr_gives() {
    let (connection, mut core) = connected().await;

    let answering = async {
        let request = core.request().await;
        let reply = GONE.replace("ID", &request["id"].to_string());
        core.send(0, DATA, reply.as_bytes()).await;
    };
    let (killed, ()) = tokio::join!(
        connection.call::<Value>("terminal.kill", json!({"terminal": "t3"})),
        answering
    );

    let error = killed.unwrap_err();
    assert!(error.is_gone(), "{error:?}");
    assert_eq!(
        error.to_string(),
        "Herdr rejected the request: t3 was removed (see `hc ls --all`)"
    );
}

#[tokio::test]
async fn streams_a_terminal_and_gives_credit_for_what_it_read() {
    let (connection, mut core) = connected().await;

    let serving = async {
        let request = core.request().await;
        core.reply(
            &request["id"],
            json!({"channel": 1, "terminal": {"address": "t3"}}),
        )
        .await;
        core.send(1, DATA, &PAINT[..20]).await;
        core.send(1, DATA, &PAINT[20..]).await;
        request["params"].clone()
    };
    let (attached, params) =
        tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);
    let mut stream = attached.unwrap();
    tokio::task::yield_now().await;

    assert_eq!(params["terminal"], "t3");
    assert_eq!(
        (params["cols"].clone(), params["rows"].clone()),
        (json!(80), json!(10))
    );
    let mut painted = stream.next().await.unwrap();
    while painted.len() < PAINT.len() {
        painted.extend(stream.next().await.unwrap());
    }
    assert_eq!(painted, PAINT);

    let mut credited = 0;
    while credited < PAINT.len() {
        let frame = core.frame().await;
        assert_eq!((frame.channel, frame.kind), (1, CREDIT));
        credited += u32::from_le_bytes(frame.payload.try_into().unwrap()) as usize;
    }
    assert_eq!(credited, PAINT.len());

    stream.input(b"echo hi\r").unwrap();
    stream.resize(100, 12).unwrap();
    assert_eq!(
        core.frame().await,
        Frame {
            channel: 1,
            kind: DATA,
            payload: b"echo hi\r".to_vec()
        }
    );
    assert_eq!(
        core.frame().await,
        Frame {
            channel: 1,
            kind: wire::RESIZE,
            payload: vec![100, 0, 12, 0]
        }
    );

    core.send(
        1,
        CLOSE,
        br#"{"reason":"exited","exit_code":0,"signal":null}"#,
    )
    .await;
    assert!(matches!(stream.next().await, Err(Error::Ended(reason)) if reason == "exited"));
}

#[tokio::test]
async fn closes_a_stream_it_drops() {
    let (connection, mut core) = connected().await;

    let serving = async {
        let request = core.request().await;
        core.reply(
            &request["id"],
            json!({"channel": 4, "terminal": {"address": "t3"}}),
        )
        .await;
    };
    let (attached, ()) = tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);
    drop(attached.unwrap());

    assert_eq!(
        core.frame().await,
        Frame {
            channel: 4,
            kind: CLOSE,
            payload: Vec::new()
        }
    );
}

#[tokio::test]
async fn fails_what_waits_once_herdr_goes_away() {
    let (connection, mut core) = connected().await;

    let leaving = async {
        core.request().await;
        drop(core);
    };
    let (listed, ()) = tokio::join!(
        connection.call::<Value>("terminal.list", json!({})),
        leaving
    );

    assert!(matches!(listed, Err(Error::Closed)));
    assert!(connection.is_closed());
    assert!(matches!(
        connection.call::<Value>("terminal.list", json!({})).await,
        Err(Error::Closed)
    ));
}

#[tokio::test]
async fn keeps_what_herdr_streams_before_it_answers() {
    let (connection, mut core) = connected().await;

    let serving = async {
        let request = core.request().await;
        core.send(1, DATA, PAINT).await;
        core.send(1, CLOSE, br#"{"reason":"exited"}"#).await;
        core.reply(
            &request["id"],
            json!({"channel": 1, "terminal": {"address": "t3"}}),
        )
        .await;
    };
    let (attached, ()) = tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);
    let mut stream = attached.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(5), stream.next());

    assert_eq!(next.await.unwrap().unwrap(), PAINT);
    assert!(matches!(stream.next().await, Err(Error::Ended(reason)) if reason == "exited"));
}

#[tokio::test(start_paused = true)]
async fn gives_up_on_a_herdr_that_stops_answering() {
    let (connection, mut core) = connected().await;

    let (listed, _) = tokio::join!(
        connection.call::<Value>("terminal.list", json!({})),
        core.request()
    );

    assert!(matches!(listed, Err(Error::Timeout)));
    assert!(connection.is_closed());
}

#[tokio::test]
async fn splits_a_large_paste_into_frames_herdr_accepts() {
    let (connection, mut core) = connected().await;
    let serving = async {
        let request = core.request().await;
        core.reply(&request["id"], json!({"channel": 2})).await;
    };
    let (attached, ()) = tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);
    let paste = vec![b'x'; 200 * 1024];

    attached.unwrap().input(&paste).unwrap();

    let mut received = Vec::new();
    while received.len() < paste.len() {
        let frame = core.frame().await;
        assert!(frame.payload.len() <= 64 * 1024);
        received.extend(frame.payload);
    }
    assert_eq!(received, paste);
}

#[tokio::test(start_paused = true)]
async fn a_request_that_times_out_ends_the_streams_on_that_connection() {
    let (connection, mut core) = connected().await;
    let serving = async {
        let request = core.request().await;
        core.reply(&request["id"], json!({"channel": 2})).await;
    };
    let (attached, ()) = tokio::join!(connection.attach("t3", 80, 10, Display::default()), serving);
    let mut stream = attached.unwrap();

    let (listed, _) = tokio::join!(
        connection.call::<Value>("terminal.list", json!({})),
        core.request()
    );

    assert!(matches!(listed, Err(Error::Timeout)));
    assert!(stream.next().await.is_err());
}
