use super::*;

#[tokio::test]
async fn a_terminal_gone_after_the_core_stops_says_why_it_ended() {
    let (stream, mut core, again) = attached().await;
    let (frames, mut queued) = mpsc::channel(4);
    let _controls = watching((Some(stream), again), frames, Launches::default());

    core.send(1, CLOSE, br#"{"reason":"server_stop"}"#).await;
    let (_, _, request) = core.frame().await;
    let request: Value = serde_json::from_slice(&request).unwrap();
    let gone = json!({"id": request["id"], "jsonrpc": "2.0", "error": {"code": -32000, "message": "no terminal \"t1\"", "data": {"kind": "not_found", "hint": "see `hc ls --all`"}}});
    core.send(0, DATA, gone.to_string().as_bytes()).await;

    let ended = wire(queued.recv().await.unwrap());
    assert_eq!(
        ended["params"]["error"],
        "Herdr rejected the request: no terminal \"t1\" (see `hc ls --all`)"
    );
}

#[tokio::test]
async fn a_lost_terminal_comes_back_on_its_new_shell() {
    let (stream, mut core, again) = attached().await;
    let (frames, mut queued) = mpsc::channel(4);
    let controls = watching((Some(stream), again), frames, Launches::default());

    core.send(
        1,
        CLOSE,
        br#"{"reason":"holder_lost","exit_code":null,"signal":null}"#,
    )
    .await;

    let (_, _, waiting) = core.frame().await;
    let waiting: Value = serde_json::from_slice(&waiting).unwrap();
    controls
        .send(Control::Resize(Size {
            cols: 100,
            rows: 30,
        }))
        .unwrap();
    let unavailable = json!({"id": waiting["id"], "jsonrpc": "2.0", "error": {"code": -32000, "message": "t1 is lost, waiting for its new shell", "data": {"kind": "unavailable", "hint": "retry after restarted"}}});
    core.send(0, DATA, unavailable.to_string().as_bytes()).await;

    let (_, _, again) = core.frame().await;
    let again: Value = serde_json::from_slice(&again).unwrap();
    assert_eq!(
        (
            again["params"]["cols"].clone(),
            again["params"]["rows"].clone()
        ),
        (json!(100), json!(30))
    );
    let reply = json!({"id": again["id"], "jsonrpc": "2.0", "result": {"channel": 2, "terminal": {"address": "t1"}}});
    core.send(0, DATA, reply.to_string().as_bytes()).await;
    core.send(2, DATA, b"PS /repos/app> ").await;

    assert_eq!(decoded(queued.recv().await.unwrap()), b"cPS /repos/app> ");
}

#[tokio::test]
async fn a_terminal_still_waiting_for_its_new_shell_is_watched_until_it_starts() {
    let (stream, mut core, again) = attached().await;
    drop(stream);
    assert_eq!(core.frame().await, (1, CLOSE, Vec::new()));
    let (frames, mut queued) = mpsc::channel(4);
    let _controls = watching((None, again), frames, Launches::default());

    let (_, _, request) = core.frame().await;
    let request: Value = serde_json::from_slice(&request).unwrap();
    assert_eq!(request["method"], "terminal.attach");
    let reply = json!({"id": request["id"], "jsonrpc": "2.0", "result": {"channel": 2, "terminal": {"address": "t1"}}});
    core.send(0, DATA, reply.to_string().as_bytes()).await;
    core.send(2, DATA, b"PS /repos/app> ").await;

    assert_eq!(decoded(queued.recv().await.unwrap()), b"cPS /repos/app> ");
}
