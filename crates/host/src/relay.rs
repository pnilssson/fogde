use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use protocol::rpc::{Failure, OFFLINE, SERVER_ERROR};
use remote::Tunnel;
use tokio_tungstenite::tungstenite;

use crate::machines;
use crate::rpc::failure;
use crate::shared::Shared;

pub async fn connect(shared: &Shared, id: &str) -> Result<(), Failure> {
    if shared
        .tunnels
        .lock()
        .await
        .get_mut(id)
        .is_some_and(Tunnel::alive)
    {
        return Ok(());
    }

    let tunnel = open(shared, id).await?;
    shared.tunnels.lock().await.insert(id.to_owned(), tunnel);

    Ok(())
}

pub async fn take(shared: &Shared, id: &str) -> Result<Tunnel, Failure> {
    if let Some(mut tunnel) = shared.tunnels.lock().await.remove(id)
        && tunnel.alive()
    {
        return Ok(tunnel);
    }

    open(shared, id).await
}

async fn open(shared: &Shared, id: &str) -> Result<Tunnel, Failure> {
    let server = machines::server(shared, id).await?;
    let binaries = shared
        .binaries
        .as_ref()
        .ok_or_else(|| Failure::new(SERVER_ERROR, "only the desktop app can reach a server"))?;
    let target = remote::Server::new(&server.target).map_err(failure)?;
    let system = remote::System {
        triple: server.triple,
    };

    target.open(&system, binaries).await.map_err(unreachable)
}

pub fn unreachable(error: remote::Error) -> Failure {
    if !error.transient() {
        return failure(error);
    }

    tracing::warn!(%error, "could not reach the server");
    Failure::new(OFFLINE, error.to_string())
}

pub async fn relay(mut page: WebSocket, tunnel: Tunnel) {
    let (mut server, mut replies) = tunnel.socket.split();

    loop {
        tokio::select! {
            message = page.recv() => match message {
                Some(Ok(Message::Text(text))) => {
                    if server.send(tungstenite::Message::text(text.as_str())).await.is_err() {
                        break;
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            message = replies.next() => match message {
                Some(Ok(tungstenite::Message::Text(text))) => {
                    if page.send(Message::text(text.as_str())).await.is_err() {
                        break;
                    }
                }
                Some(Ok(tungstenite::Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
        }
    }
}
