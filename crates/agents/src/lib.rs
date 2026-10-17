mod agent;
mod error;
mod status;

pub use agent::Agent;
pub use error::Error;

pub async fn signed_in() -> Result<Vec<Agent>, Error> {
    let (claude, codex) = tokio::join!(Agent::Claude.signed_in(), Agent::Codex.signed_in());

    Ok([(Agent::Claude, claude?), (Agent::Codex, codex?)]
        .into_iter()
        .filter_map(|(agent, signed_in)| signed_in.then_some(agent))
        .collect())
}
