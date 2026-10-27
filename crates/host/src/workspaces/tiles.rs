use std::path::Path;

use protocol::rpc::Failure;
use protocol::workspaces::{MoveTile, MoveTileParams};

use super::{pool, store, watch};
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<MoveTile>();
}

pub async fn close_terminals_in(session: &Session, checkout: &Path) -> Result<(), Failure> {
    let terminals = store::terminals_in(pool(session).await?, checkout)
        .await
        .map_err(failure)?;

    crate::terminals::close(session, &terminals).await
}

impl Handler for MoveTile {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: MoveTileParams) -> Result<(), Failure> {
        store::arrange(pool(session).await?, &params.tile, params.before.as_deref())
            .await
            .map_err(failure)?;
        watch::announce(&session.shared);

        Ok(())
    }
}
