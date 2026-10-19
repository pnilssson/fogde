mod sign_ins;

use agents::{Prompt, SignIn};
use protocol::agents::{
    Agent, AgentPrompt, AgentState, AgentStatus, CancelAgentSignIn, FinishAgentSignIn,
    FinishAgentSignInParams, InstallAgent, ListAgents, StartAgentSignIn,
};
use protocol::rpc::{Failure, SERVER_ERROR};
use tokio::sync::Mutex;

use crate::rpc::{Handler, Router, cancelled, failure};
use crate::session::Session;
use sign_ins::SignIns;

#[derive(Default)]
pub struct State {
    sign_ins: Mutex<SignIns>,
}

pub fn register(router: &mut Router) {
    router.add::<ListAgents>();
    router.add::<StartAgentSignIn>();
    router.add::<FinishAgentSignIn>();
    router.add::<CancelAgentSignIn>();
    router.add::<InstallAgent>();
}

impl Handler for ListAgents {
    async fn handle(session: &Session, (): ()) -> Result<Vec<AgentStatus>, Failure> {
        let states = agents::states().await.map_err(failure)?;
        let github = !states
            .iter()
            .any(|(agent, state)| agent.uses_github() && *state == agents::State::SignedIn)
            || github(session).await?;

        Ok(states
            .into_iter()
            .map(|(agent, state)| AgentStatus {
                agent: to_protocol(agent),
                state: match state {
                    agents::State::Missing => AgentState::Missing,
                    agents::State::SignedIn if github || !agent.uses_github() => {
                        AgentState::SignedIn
                    }
                    agents::State::SignedIn | agents::State::SignedOut => AgentState::SignedOut,
                },
            })
            .collect())
    }
}

impl Handler for StartAgentSignIn {
    async fn handle(session: &Session, agent: Agent) -> Result<AgentPrompt, Failure> {
        let agent = from_protocol(agent);
        if agent.uses_github() && !github(session).await? {
            return Err(Failure::new(
                SERVER_ERROR,
                format!("{agent} signs in through GitHub, so set up GitHub first"),
            ));
        }
        let (id, stop) = session.agents.sign_ins.lock().await.begin(agent);

        let started = tokio::select! {
            started = SignIn::start(agent, !session.local) => started,
            _ = stop => return Err(cancelled()),
            () = session.closed() => return Err(cancelled()),
        };

        let mut sign_ins = session.agents.sign_ins.lock().await;
        let signin = match started {
            Ok(Some(signin)) => signin,
            Ok(None) => {
                sign_ins.end(agent, id);
                return Ok(AgentPrompt::Done);
            }
            Err(error) => {
                sign_ins.end(agent, id);
                return Err(failure(error));
            }
        };
        let prompt = match &signin.prompt {
            Prompt::Browser => AgentPrompt::Browser,
            Prompt::Code { code, url } => AgentPrompt::Code {
                code: code.clone(),
                url: url.clone(),
            },
            Prompt::Paste { url } => AgentPrompt::Paste { url: url.clone() },
        };
        let waiting = sign_ins.wait(agent, id, signin);
        drop(sign_ins);

        if !waiting {
            return Err(cancelled());
        }

        Ok(prompt)
    }
}

impl Handler for FinishAgentSignIn {
    async fn handle(session: &Session, params: FinishAgentSignInParams) -> Result<(), Failure> {
        let agent = from_protocol(params.agent);
        let Some((id, signin, stop)) = session.agents.sign_ins.lock().await.resume(agent) else {
            return Err(Failure::new(SERVER_ERROR, "no sign-in is in progress"));
        };

        let result = tokio::select! {
            result = signin.finish(params.code.as_deref()) => result,
            _ = stop => return Err(cancelled()),
            () = session.closed() => return Err(cancelled()),
        };
        session.agents.sign_ins.lock().await.end(agent, id);

        result.map_err(failure)
    }
}

impl Handler for CancelAgentSignIn {
    async fn handle(session: &Session, agent: Agent) -> Result<(), Failure> {
        session
            .agents
            .sign_ins
            .lock()
            .await
            .cancel(from_protocol(agent));

        Ok(())
    }
}

impl Handler for InstallAgent {
    async fn handle(_: &Session, agent: Agent) -> Result<(), Failure> {
        from_protocol(agent).install().await.map_err(failure)
    }
}

async fn github(session: &Session) -> Result<bool, Failure> {
    let accounts = session.gh().await?.accounts().await.map_err(failure)?;

    Ok(!accounts.is_empty())
}

fn to_protocol(agent: agents::Agent) -> Agent {
    match agent {
        agents::Agent::Claude => Agent::Claude,
        agents::Agent::Codex => Agent::Codex,
        agents::Agent::Copilot => Agent::Copilot,
    }
}

fn from_protocol(agent: Agent) -> agents::Agent {
    match agent {
        Agent::Claude => agents::Agent::Claude,
        Agent::Codex => agents::Agent::Codex,
        Agent::Copilot => agents::Agent::Copilot,
    }
}
