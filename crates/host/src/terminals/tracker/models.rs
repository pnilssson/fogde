use std::collections::HashSet;

use futures_util::future::join_all;
use herdr_client::{Client, Pane};
use protocol::agents::Agent;

use super::{Tracker, lock};
use crate::terminals::launch;

#[derive(Clone)]
pub(super) struct Model {
    pub(super) agent: Agent,
    pub(super) status: herdr_client::Status,
    pub(super) name: Option<String>,
}

impl Tracker {
    pub(super) async fn read_models(&self, client: &Client, panes: &[Pane]) -> bool {
        let known = lock(&self.models).clone();
        let mut workspaces = HashSet::new();
        let read = panes
            .iter()
            .filter(|pane| workspaces.insert(&pane.workspace))
            .filter_map(|pane| {
                let agent = pane.agent.as_deref().and_then(launch::agent)?;
                if known
                    .get(&pane.workspace)
                    .is_some_and(|model| model.agent == agent && model.status == pane.status)
                {
                    return None;
                }

                Some(async move {
                    let name = match client.processes(&pane.id).await {
                        Ok(processes) => {
                            let foreground = processes.iter().map(|process| process.pid).collect();
                            agents::model(agent, foreground).await
                        }
                        Err(error) => {
                            tracing::warn!(%error, pane = %pane.id, "could not read the agent's processes");
                            None
                        }
                    };

                    (
                        pane.workspace.clone(),
                        Model {
                            agent,
                            status: pane.status,
                            name,
                        },
                    )
                })
            });
        let read = join_all(read).await;

        let changed = read
            .iter()
            .any(|(id, model)| known.get(id).is_none_or(|known| known.name != model.name));

        let mut models = lock(&self.models);
        models.retain(|id, _| {
            panes
                .iter()
                .find(|pane| &pane.workspace == id)
                .is_some_and(|pane| pane.agent.is_some())
        });
        models.extend(read);

        changed
    }
}
