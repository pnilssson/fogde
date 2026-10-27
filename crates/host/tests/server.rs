use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Error, Message};

async fn exchange(url: &str, message: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let (mut socket, _) = connect_async(url).await?;
    socket.send(Message::text(message)).await?;

    loop {
        let reply = socket.next().await.ok_or("socket closed")??;
        let value: Value = serde_json::from_str(reply.to_text()?)?;
        if value.get("method").is_none() {
            return Ok(value);
        }
    }
}

#[tokio::test]
async fn rejects_wrong_token() {
    let connection = host::start(host::Binaries::built(std::env::temp_dir()), None)
        .await
        .unwrap();

    let result = connect_async(format!("{}?token=wrong", connection.url)).await;

    assert!(matches!(result, Err(Error::Http(response)) if response.status() == 401));
}

#[tokio::test]
async fn rejects_missing_token() {
    let connection = host::start(host::Binaries::built(std::env::temp_dir()), None)
        .await
        .unwrap();

    assert!(connect_async(connection.url).await.is_err());
}

#[tokio::test]
async fn answers_unknown_method_and_bad_json() {
    let connection = host::start(host::Binaries::built(std::env::temp_dir()), None)
        .await
        .unwrap();
    let url = format!("{}?token={}", connection.url, connection.token);

    let unknown = exchange(&url, r#"{"jsonrpc":"2.0","id":7,"method":"nope"}"#)
        .await
        .unwrap();
    let garbage = exchange(&url, "not json").await.unwrap();

    assert_eq!(unknown["id"], json!(7));
    assert_eq!(unknown["error"]["code"], json!(-32601));
    assert_eq!(garbage["id"], Value::Null);
    assert_eq!(garbage["error"]["code"], json!(-32700));
}
