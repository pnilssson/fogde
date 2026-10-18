mod agent;
mod error;
mod install;
mod status;

pub use agent::{Agent, State};
pub use error::Error;

const ALL: [Agent; 3] = [Agent::Claude, Agent::Codex, Agent::Copilot];

pub async fn states() -> Result<Vec<(Agent, State)>, Error> {
    let (claude, codex, copilot) = tokio::join!(
        Agent::Claude.state(),
        Agent::Codex.state(),
        Agent::Copilot.state()
    );

    Ok(ALL.into_iter().zip([claude?, codex?, copilot?]).collect())
}
