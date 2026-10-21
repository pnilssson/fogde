use std::io;
use std::path::{Path, PathBuf};

use interprocess::local_socket::Name;
use interprocess::local_socket::tokio::{RecvHalf, SendHalf, Stream, prelude::*};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};

use crate::Error;

const ID: &str = "fogde";

#[derive(Clone, Debug)]
pub struct Client {
    socket: PathBuf,
}

impl Client {
    pub fn new(socket: &Path) -> Self {
        Self {
            socket: socket.to_owned(),
        }
    }

    pub(crate) async fn request<R: DeserializeOwned>(
        &self,
        method: &str,
        params: impl Serialize,
    ) -> Result<R, Error> {
        let mut connection = self.open(method, params).await?;

        reply(&connection.line().await?)
    }

    pub(crate) async fn open(
        &self,
        method: &str,
        params: impl Serialize,
    ) -> Result<Connection, Error> {
        let stream = Stream::connect(name(&self.socket).map_err(Error::Connect)?)
            .await
            .map_err(Error::Connect)?;
        let (reader, mut writer) = stream.split();

        let mut line = serde_json::to_vec(&Request {
            id: ID,
            method,
            params,
        })?;
        line.push(b'\n');
        writer.write_all(&line).await?;

        Ok(Connection {
            lines: BufReader::new(reader).lines(),
            _writer: writer,
        })
    }
}

pub(crate) struct Connection {
    lines: Lines<BufReader<RecvHalf>>,
    _writer: SendHalf,
}

impl Connection {
    pub(crate) async fn line(&mut self) -> Result<String, Error> {
        self.lines.next_line().await?.ok_or(Error::Closed)
    }
}

pub(crate) fn reply<R: DeserializeOwned>(line: &str) -> Result<R, Error> {
    let response: Response<R> = serde_json::from_str(line)?;
    match response.outcome {
        Outcome::Error(failure) => Err(Error::Herdr {
            code: failure.code,
            message: failure.message,
        }),
        Outcome::Result(_) if response.id != ID => Err(Error::Mismatch {
            expected: ID.to_owned(),
            actual: response.id,
        }),
        Outcome::Result(result) => Ok(result),
    }
}

#[cfg(unix)]
fn name(socket: &Path) -> io::Result<Name<'_>> {
    use interprocess::local_socket::GenericFilePath;

    socket.to_fs_name::<GenericFilePath>()
}

#[cfg(windows)]
fn name(socket: &Path) -> io::Result<Name<'_>> {
    use interprocess::local_socket::GenericNamespaced;

    socket.as_os_str().to_ns_name::<GenericNamespaced>()
}

#[derive(Serialize)]
pub(crate) struct Empty {}

#[derive(Serialize)]
struct Request<'a, P> {
    id: &'a str,
    method: &'a str,
    params: P,
}

#[derive(Deserialize)]
struct Response<R> {
    id: String,
    #[serde(flatten)]
    outcome: Outcome<R>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Outcome<R> {
    Result(R),
    Error(Failure),
}

#[derive(Deserialize)]
struct Failure {
    code: String,
    message: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;

    use interprocess::local_socket::ListenerOptions;
    use serde_json::{Value, json};

    use super::*;

    pub(crate) fn serve_each(test: &str, replies: Vec<String>) -> PathBuf {
        let socket = std::env::temp_dir().join(format!("fogde-{test}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&socket);
        let listener = ListenerOptions::new()
            .name(name(&socket).unwrap())
            .create_tokio()
            .unwrap();

        tokio::spawn(async move {
            for reply in replies {
                let (reader, mut writer) = listener.accept().await.unwrap().split();
                let mut request = String::new();
                BufReader::new(reader)
                    .read_line(&mut request)
                    .await
                    .unwrap();
                writer
                    .write_all(format!("{reply}\n").as_bytes())
                    .await
                    .unwrap();
            }
        });

        socket
    }

    fn serve(test: &str, reply: Option<&'static str>) -> PathBuf {
        let socket = std::env::temp_dir().join(format!("fogde-{test}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&socket);
        let listener = ListenerOptions::new()
            .name(name(&socket).unwrap())
            .create_tokio()
            .unwrap();

        tokio::spawn(async move {
            let (reader, mut writer) = listener.accept().await.unwrap().split();
            let mut request = String::new();
            BufReader::new(reader)
                .read_line(&mut request)
                .await
                .unwrap();

            if let Some(reply) = reply {
                writer
                    .write_all(format!("{reply}\n").as_bytes())
                    .await
                    .unwrap();
            }
        });

        socket
    }

    async fn ping(socket: &Path) -> Result<Value, Error> {
        Client::new(socket).request("ping", json!({})).await
    }

    #[tokio::test]
    async fn returns_result() {
        let socket = serve("result", Some(r#"{"id":"fogde","result":{"type":"pong"}}"#));

        assert_eq!(ping(&socket).await.unwrap(), json!({"type": "pong"}));
    }

    #[tokio::test]
    async fn returns_herdr_error() {
        let socket = serve(
            "error",
            Some(r#"{"id":"fogde","error":{"code":"not_found","message":"pane not found"}}"#),
        );

        assert!(matches!(
            ping(&socket).await,
            Err(Error::Herdr { code, .. }) if code == "not_found"
        ));
    }

    #[tokio::test]
    async fn rejects_other_request_id() {
        let socket = serve("mismatch", Some(r#"{"id":"7","result":{"type":"pong"}}"#));

        assert!(matches!(ping(&socket).await, Err(Error::Mismatch { .. })));
    }

    #[tokio::test]
    async fn reports_closed_connection() {
        let socket = serve("closed", None);

        assert!(matches!(ping(&socket).await, Err(Error::Closed)));
    }

    #[tokio::test]
    async fn reports_missing_server() {
        let socket = std::env::temp_dir().join("fogde-missing.sock");

        assert!(matches!(ping(&socket).await, Err(Error::Connect(_))));
    }
}
