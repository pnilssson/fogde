use std::collections::BTreeMap;
use std::sync::Arc;

use herdr_client::{Connection, Terminal};
use protocol::terminals::Status;

use super::describe::{seen, status};
use super::{SEEN, Tracker, lock};

impl Tracker {
    pub(super) fn mark(&self, client: &Arc<Connection>, terminals: &BTreeMap<String, Terminal>) {
        let viewed = self.viewed();
        let mut marked = lock(&self.marked);
        marked.retain(|id, _| terminals.contains_key(id));

        for id in viewed {
            let Some(terminal) = terminals.get(&id) else {
                continue;
            };
            let busy = matches!(status(terminal), Status::Working | Status::Blocked);
            if busy
                || seen(terminal).is_some_and(|seen| seen >= terminal.turn)
                || marked.get(&id) == Some(&terminal.turn)
            {
                continue;
            }

            marked.insert(id.clone(), terminal.turn);
            let client = Arc::clone(client);
            let labels = BTreeMap::from([(SEEN.to_owned(), terminal.turn.to_string())]);
            tokio::spawn(async move {
                if let Err(error) = client.label(&id, &labels).await {
                    tracing::warn!(%error, %id, "could not mark the agent as seen");
                }
            });
        }
    }
}
