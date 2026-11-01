use std::time::Duration;

use herdr_client::Connection;
use protocol::agents::Agent;
use serde_json::{Value, json};
use tokio::io::{
    AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf,
};

use super::*;

const HELLO: &str = r#"{"id":0,"jsonrpc":"2.0","result":{"core":{"id":"45255384ce5fc955fddc39260e47956f","name":"dev-pc"},"features":["framed","attach","events"],"framed":true,"generation":1,"methods":["hello","terminal.attach"],"version":"0.1.16"}}"#;
const DATA: u8 = 0;
const CREDIT: u8 = 1;
const CLOSE: u8 = 2;

struct Core {
    reader: BufReader<ReadHalf<DuplexStream>>,
    writer: WriteHalf<DuplexStream>,
}

impl Core {
    async fn frame(&mut self) -> (u32, u8, Vec<u8>) {
        let mut header = [0; 9];
        self.reader.read_exact(&mut header).await.unwrap();
        let length = u32::from_le_bytes([header[5], header[6], header[7], header[8]]);
        let mut payload = vec![0; length as usize];
        self.reader.read_exact(&mut payload).await.unwrap();

        (
            u32::from_le_bytes([header[0], header[1], header[2], header[3]]),
            header[4],
            payload,
        )
    }

    async fn send(&mut self, channel: u32, kind: u8, payload: &[u8]) {
        let mut frame = channel.to_le_bytes().to_vec();
        frame.push(kind);
        frame.extend(u32::try_from(payload.len()).unwrap().to_le_bytes());
        frame.extend(payload);
        self.writer.write_all(&frame).await.unwrap();
    }
}

async fn attached() -> (Stream, Core) {
    let (ours, theirs) = tokio::io::duplex(1 << 20);
    let (reader, writer) = tokio::io::split(ours);
    let (reader_core, writer_core) = tokio::io::split(theirs);
    let mut core = Core {
        reader: BufReader::new(reader_core),
        writer: writer_core,
    };

    let greet = async {
        let mut line = String::new();
        core.reader.read_line(&mut line).await.unwrap();
        core.writer
            .write_all(format!("{HELLO}\n").as_bytes())
            .await
            .unwrap();
    };
    let (connection, ()) = tokio::join!(Connection::open(reader, writer, None), greet);
    let connection = connection.unwrap();

    let serve = async {
        let (_, _, request) = core.frame().await;
        let request: Value = serde_json::from_slice(&request).unwrap();
        let reply = json!({"id": request["id"], "jsonrpc": "2.0", "result": {"channel": 1, "terminal": {"address": "t1"}}});
        core.send(0, DATA, reply.to_string().as_bytes()).await;
    };
    let (stream, ()) = tokio::join!(connection.attach("t1", 80, 24), serve);

    (stream.unwrap(), core)
}

fn watching(
    stream: Stream,
    frames: mpsc::Sender<Message>,
    launches: Launches,
) -> mpsc::UnboundedSender<Control> {
    let (controls, receiver) = mpsc::unbounded_channel();
    tokio::spawn(watch(stream, "t1".into(), 7, receiver, frames, launches));

    controls
}

fn wire(message: Message) -> Value {
    serde_json::to_value(message).unwrap()
}

fn decoded(message: Message) -> Vec<u8> {
    let message = wire(message);
    assert_eq!(message["method"], "terminals.output");
    assert_eq!(message["params"]["token"], 7);

    STANDARD
        .decode(message["params"]["data"].as_str().unwrap())
        .unwrap()
}

#[tokio::test]
async fn the_first_output_starts_the_agent_it_was_opened_for() {
    let (stream, mut core) = attached().await;
    let (frames, mut queued) = mpsc::channel(4);
    let launches = Launches::default();
    launches.hold("t1".into(), Agent::Codex);
    let _controls = watching(stream, frames, launches.clone());

    core.send(1, DATA, b"PS /repos/app> ").await;

    assert_eq!(decoded(queued.recv().await.unwrap()), b"PS /repos/app> ");
    assert_eq!(
        core.frame().await,
        (1, CREDIT, 15u32.to_le_bytes().to_vec())
    );
    assert_eq!(
        core.frame().await,
        (1, DATA, b"codex --no-daemon\r".to_vec())
    );
    assert!(launches.take("t1").is_none());
}

#[tokio::test]
async fn a_client_that_falls_behind_holds_herdr_back_without_losing_output() {
    let (stream, mut core) = attached().await;
    let (frames, mut queued) = mpsc::channel(1);
    let other = notification::<EndedEvent>(TerminalEnded {
        id: "t2".into(),
        error: None,
    });
    frames.send(other.unwrap()).await.unwrap();
    let _controls = watching(stream, frames, Launches::default());

    core.send(1, DATA, b"one ").await;
    assert_eq!(core.frame().await, (1, CREDIT, 4u32.to_le_bytes().to_vec()));
    core.send(1, DATA, b"two ").await;
    core.send(1, DATA, b"three").await;
    let early = tokio::time::timeout(Duration::from_millis(100), core.frame()).await;
    assert!(
        early.is_err(),
        "credit went out before the client caught up"
    );

    queued.recv().await.unwrap();
    assert_eq!(decoded(queued.recv().await.unwrap()), b"one ");
    assert_eq!(core.frame().await, (1, CREDIT, 9u32.to_le_bytes().to_vec()));
    assert_eq!(decoded(queued.recv().await.unwrap()), b"two three");
}

#[tokio::test]
async fn a_program_that_exits_ends_the_terminal_without_an_error() {
    let (stream, mut core) = attached().await;
    let (frames, mut queued) = mpsc::channel(4);
    let _controls = watching(stream, frames, Launches::default());

    core.send(1, DATA, b"bye").await;
    core.send(
        1,
        CLOSE,
        br#"{"reason":"exited","exit_code":0,"signal":null}"#,
    )
    .await;

    assert_eq!(decoded(queued.recv().await.unwrap()), b"bye");
    let ended = wire(queued.recv().await.unwrap());
    assert_eq!(ended["method"], "terminals.ended");
    assert_eq!(ended["params"], json!({"id": "t1", "error": null}));
}

#[tokio::test]
async fn a_core_that_stops_says_why_the_terminal_ended() {
    let (stream, mut core) = attached().await;
    let (frames, mut queued) = mpsc::channel(4);
    let _controls = watching(stream, frames, Launches::default());

    core.send(1, CLOSE, br#"{"reason":"server_stop"}"#).await;

    let ended = wire(queued.recv().await.unwrap());
    assert_eq!(ended["params"]["error"], "the terminal ended: server_stop");
}

#[tokio::test]
async fn typing_and_resizing_reach_the_program() {
    let (stream, mut core) = attached().await;
    let (frames, _queued) = mpsc::channel(4);
    let controls = watching(stream, frames, Launches::default());

    controls.send(Control::Input(b"ls\r".to_vec())).unwrap();
    controls
        .send(Control::Resize(Size {
            cols: 100,
            rows: 30,
        }))
        .unwrap();

    assert_eq!(core.frame().await, (1, DATA, b"ls\r".to_vec()));
    assert_eq!(core.frame().await, (1, 3, vec![100, 0, 30, 0]));
    drop(controls);
    assert_eq!(core.frame().await, (1, CLOSE, Vec::new()));
}
