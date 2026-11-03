use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, ReadHalf, WriteHalf};

use crate::Connection;
use crate::wire::{self, DATA, Frame};

pub(crate) const HELLO: &str = r#"{"id":0,"jsonrpc":"2.0","result":{"core":{"id":"45255384ce5fc955fddc39260e47956f","name":"dev-pc"},"features":["framed","attach","events","status","shell"],"framed":true,"generation":1,"methods":["hello","terminal.run","terminal.list","terminal.kill","terminal.attach","events.subscribe"],"version":"0.1.16"}}"#;

pub(crate) struct Core {
    reader: BufReader<ReadHalf<DuplexStream>>,
    writer: WriteHalf<DuplexStream>,
}

impl Core {
    pub(crate) async fn frame(&mut self) -> Frame {
        wire::read(&mut self.reader).await.unwrap().unwrap()
    }

    pub(crate) async fn request(&mut self) -> Value {
        let frame = self.frame().await;
        assert_eq!((frame.channel, frame.kind), (0, DATA));

        serde_json::from_slice(&frame.payload).unwrap()
    }

    pub(crate) async fn send(&mut self, channel: u32, kind: u8, payload: &[u8]) {
        self.writer
            .write_all(&wire::encode(channel, kind, payload))
            .await
            .unwrap();
    }

    pub(crate) async fn reply(&mut self, id: &Value, result: Value) {
        let reply = json!({"id": id, "jsonrpc": "2.0", "result": result});
        self.send(0, DATA, reply.to_string().as_bytes()).await;
    }
}

pub(crate) async fn connected() -> (Connection, Core) {
    greeted(HELLO).await
}

pub(crate) async fn greeted(hello: &str) -> (Connection, Core) {
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
            .write_all(format!("{hello}\n").as_bytes())
            .await
            .unwrap();
        line
    };
    let (connection, sent) = tokio::join!(Connection::open(reader, writer, None), greet);

    assert!(sent.contains(r#""method":"hello""#) && sent.contains(r#""framed":true"#));
    (connection.unwrap(), core)
}
