use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use protocol::Connection;
use protocol::rpc::{self, Failure, PARSE_ERROR};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use remote::Binaries;

use crate::relay;
use crate::session::Session;
use crate::shared::Shared;
use crate::token;

struct Desktop {
    shared: Arc<Shared>,
    token: String,
}

#[derive(Deserialize)]
struct Auth {
    token: String,
}

pub async fn start(binaries: Binaries) -> io::Result<Connection> {
    let token = token::generate()?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let url = format!("ws://{}/", listener.local_addr()?);

    let desktop = Arc::new(Desktop {
        shared: Arc::new(Shared::new(Some(binaries))?),
        token: token.clone(),
    });

    let app = Router::new()
        .route("/", get(upgrade))
        .route("/server", get(server))
        .with_state(desktop)
        .into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::error!(%error, "host stopped");
        }
    });

    Ok(Connection { url, token })
}

async fn upgrade(
    upgrade: WebSocketUpgrade,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(auth): Query<Auth>,
    State(desktop): State<Arc<Desktop>>,
) -> Response {
    if !token::matches(&auth.token, &desktop.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let local = peer.ip().is_loopback();
    let shared = Arc::clone(&desktop.shared);
    upgrade.on_upgrade(move |socket| serve(socket, shared, local))
}

async fn server(
    upgrade: WebSocketUpgrade,
    Query(auth): Query<Auth>,
    State(desktop): State<Arc<Desktop>>,
) -> Response {
    if !token::matches(&auth.token, &desktop.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match relay::open(&desktop.shared).await {
        Ok(tunnel) => upgrade.on_upgrade(move |socket| relay::relay(socket, tunnel)),
        Err(error) => {
            tracing::warn!(%error, "could not reach the server");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub(crate) async fn serve(mut socket: WebSocket, shared: Arc<Shared>, local: bool) {
    let session = Arc::new(Session::new(shared, local));
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

    session.close();
}
