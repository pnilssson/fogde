mod agent;
mod error;
mod install;
pub mod limits;
mod signin;
mod status;

use futures_util::future::try_join_all;
use protocol::agents::Agent;

pub use agent::{State, install, state};
pub use error::Error;
pub use signin::{Prompt, SignIn};

pub async fn states() -> Result<Vec<(Agent, State)>, Error> {
    let states = try_join_all(Agent::ALL.map(state)).await?;

    Ok(Agent::ALL.into_iter().zip(states).collect())
}
