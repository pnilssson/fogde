use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Connection {
    pub url: String,
    pub token: String,
}
