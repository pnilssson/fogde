use std::fmt::Write as _;

use protocol::Connection;
use protocol::server::ConnectServer;

use crate::Client;

impl Client {
    pub fn server(own: Self, connection: Connection, id: String) -> Self {
        Self::new(move || {
            let own = own.clone();
            let connection = connection.clone();
            let id = id.clone();
            async move {
                own.call::<ConnectServer>(id.clone()).await?;

                Ok(Connection {
                    url: format!("{}servers/{}", connection.url, encode(&id)),
                    token: connection.token,
                })
            }
        })
    }
}

fn encode(text: &str) -> String {
    text.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
        encoded
    })
}
