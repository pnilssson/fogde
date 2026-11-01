use std::collections::BTreeMap;

use futures_util::future::join_all;
use herdr_client::Terminal;
use protocol::agents::Agent;
use protocol::terminals::Status;

use super::describe::{running, status};
use super::{Tracker, lock};

#[derive(Clone)]
pub(super) struct Model {
    pub(super) agent: Agent,
    pub(super) status: Status,
    pub(super) name: Option<String>,
}

impl Tracker {
    pub(super) async fn read_models(&self, terminals: &BTreeMap<String, Terminal>) -> bool {
        let known = lock(&self.models).clone();
        let read = terminals.values().filter_map(|terminal| {
            let agent = running(terminal)?;
            let status = status(terminal);
            if known
                .get(&terminal.address)
                .is_some_and(|model| model.agent == agent && model.status == status)
            {
                return None;
            }

            let foreground = terminal
                .foreground
                .as_ref()
                .and_then(|process| u32::try_from(process.pid).ok());
            Some(async move {
                let name = match foreground {
                    Some(pid) => agents::model(agent, vec![pid]).await,
                    None => None,
                };

                (
                    terminal.address.clone(),
                    Model {
                        agent,
                        status,
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
        models.retain(|id, _| terminals.get(id).and_then(running).is_some());
        models.extend(read);

        changed
    }
}
