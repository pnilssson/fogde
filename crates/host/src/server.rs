use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use protocol::Connection;
use protocol::rpc::{self, Failure, PARSE_ERROR};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::session::Session;
use crate::shared::Shared;
use crate::token;

#[derive(Deserialize)]
struct Auth {
    token: String,
}

pub async fn start() -> io::Result<Connection> {
    let token = token::generate()?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let url = format!("ws://{}/", listener.local_addr()?);

    let shared = Arc::new(Shared::new(token.clone())?);

    let app = Router::new().route("/", get(upgrade)).with_state(shared);
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::error!(%error, "host stopped");
        }
    });

    Ok(Connection { url, token })
}

async fn upgrade(
    upgrade: WebSocketUpgrade,
    Query(auth): Query<Auth>,
    State(shared): State<Arc<Shared>>,
) -> Response {
    if !token::matches(&auth.token, &shared.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    upgrade.on_upgrade(|socket| serve(socket, shared))
}

async fn serve(mut socket: WebSocket, shared: Arc<Shared>) {
    let session = Arc::new(Session::new(shared));
    let (replies, mut outgoing) = mpsc::unbounded_channel::<rpc::Response>();

    loop {
        tokio::select! {
            message = socket.recv() => {
                let Some(Ok(message)) = message else {
                    break;
                };
                let Message::Text(text) = message else {
                    continue;
                };

                match serde_json::from_str::<rpc::Request>(&text) {
                    Ok(request) => {
                        let session = Arc::clone(&session);
                        let replies = replies.clone();
                        tokio::spawn(async move {
                            let _ = replies.send(session.handle(request).await);
                        });
                    }
                    Err(error) => {
                        let _ = replies.send(rpc::Response::new(
                            None,
                            Err(Failure::new(PARSE_ERROR, error.to_string())),
                        ));
                    }
                }
            }
            Some(reply) = outgoing.recv() => {
                let Ok(text) = serde_json::to_string(&reply) else {
                    continue;
                };
                if socket.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
        }
    }
}
