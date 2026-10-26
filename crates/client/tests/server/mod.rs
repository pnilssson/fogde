use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use futures_util::{SinkExt, StreamExt};
use protocol::Connection;
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, accept_async};

#[derive(Default)]
pub struct Options {
    pub refuse: usize,
    pub stall: bool,
    pub error: Option<String>,
    pub hold: bool,
}

pub struct Server {
    pub url: String,
    pub options: Mutex<Options>,
    pub sockets: AtomicUsize,
    pub requests: Mutex<Vec<String>>,
    open: Mutex<Vec<mpsc::UnboundedSender<Message>>>,
}

impl Server {
    pub async fn start(name: &'static str) -> io::Result<Arc<Self>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let server = Arc::new(Self {
            url: format!("ws://{}/", listener.local_addr()?),
            options: Mutex::default(),
            sockets: AtomicUsize::new(0),
            requests: Mutex::default(),
            open: Mutex::default(),
        });

        let accepting = server.clone();
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    return;
                };
                accepting.sockets.fetch_add(1, Ordering::SeqCst);
                let (refuse, stall) = {
                    let mut options = lock(&accepting.options);
                    let refuse = options.refuse > 0;
                    options.refuse = options.refuse.saturating_sub(1);
                    (refuse, options.stall)
                };

                if refuse {
                    drop(stream);
                    continue;
                }

                if stall {
                    tokio::spawn(async move {
                        let _stream = stream;
                        std::future::pending::<()>().await;
                    });
                    continue;
                }

                let Ok(socket) = accept_async(stream).await else {
                    continue;
                };
                tokio::spawn(accepting.clone().serve(socket, name));
            }
        });

        Ok(server)
    }

    pub fn connection(&self) -> Connection {
        Connection {
            url: self.url.clone(),
            token: "test".to_owned(),
        }
    }

    pub fn push(&self, method: &str, params: &Value) {
        let text = json!({"jsonrpc": "2.0", "method": method, "params": params}).to_string();
        for socket in lock(&self.open).iter() {
            let _ = socket.send(Message::text(text.clone()));
        }
    }

    pub fn drop_sockets(&self) {
        for socket in lock(&self.open).drain(..) {
            let _ = socket.send(Message::Close(None));
        }
    }

    async fn serve(self: Arc<Self>, socket: WebSocketStream<TcpStream>, name: &'static str) {
        let (mut sink, mut stream) = socket.split();
        let (outgoing, mut queue) = mpsc::unbounded_channel();
        lock(&self.open).push(outgoing.clone());
        tokio::spawn(async move {
            while let Some(message) = queue.recv().await {
                let close = matches!(message, Message::Close(_));
                if sink.send(message).await.is_err() || close {
                    break;
                }
            }
        });

        while let Some(Ok(Message::Text(text))) = stream.next().await {
            let Ok(request) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let method = request["method"].as_str().unwrap_or_default().to_owned();
            lock(&self.requests).push(method.clone());

            let (hold, error) = {
                let options = lock(&self.options);
                (options.hold && method == "test.hold", options.error.clone())
            };
            if hold {
                continue;
            }

            let reply = match error {
                Some(message) => json!({
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {"code": -32000, "message": message},
                }),
                None if method == "server.connect" => {
                    json!({"jsonrpc": "2.0", "id": request["id"], "result": null})
                }
                None => json!({"jsonrpc": "2.0", "id": request["id"], "result": name}),
            };
            let _ = outgoing.send(Message::text(reply.to_string()));
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
