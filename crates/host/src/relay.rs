use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use remote::Tunnel;
use tokio_tungstenite::tungstenite;

use crate::shared::Shared;

pub async fn open(shared: &Shared) -> Result<Tunnel, String> {
    if let Some(mut tunnel) = shared.tunnel.lock().await.take()
        && tunnel.alive()
    {
        return Ok(tunnel);
    }

    let server = shared
        .settings
        .server()
        .await
        .map_err(|error| error.to_string())?
        .ok_or("no server is set up")?;
    let binaries = shared.binaries.as_ref().ok_or("no host builds")?;
    let target = remote::Server::new(&server.target).map_err(|error| error.to_string())?;
    let system = remote::System {
        triple: server.triple,
    };

    target
        .open(&system, binaries)
        .await
        .map_err(|error| error.to_string())
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
