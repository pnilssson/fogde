use serde::Deserialize;

use crate::Error;
use crate::gh::{Gh, failure};

const AUTH_REQUIRED: i32 = 4;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

impl Gh {
    pub async fn user(&self) -> Result<Option<User>, Error> {
        let output = self.run(&["api", "user"]).await?;
        if output.status.code() == Some(AUTH_REQUIRED) {
            return Ok(None);
        }

        if !output.status.success() {
            return Err(failure(&output.stderr));
        }

        Ok(Some(serde_json::from_slice(&output.stdout)?))
    }
}
