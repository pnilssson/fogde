use std::path::Path;

use protocol::rpc::Failure;
use protocol::workspaces::{
    CloseTile, MoveTile, MoveTileParams, OpenMonitor, OpenMonitorParams, OpenNote, OpenNoteParams,
    ShowNote, ShowNoteParams,
};

use super::{pool, store, watch};
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<MoveTile>();
    router.add::<OpenNote>();
    router.add::<ShowNote>();
    router.add::<OpenMonitor>();
    router.add::<CloseTile>();
}

pub async fn close_in(session: &Session, checkout: &Path) -> Result<(), Failure> {
    let closed = store::close_in(pool(session).await?, checkout)
        .await
        .map_err(failure)?;
    if closed {
        watch::announce(&session.shared);
    }

    Ok(())
}

pub async fn follow(session: &Session, from: &Path, to: &Path) -> Result<(), Failure> {
    let followed = store::follow(pool(session).await?, from, to)
        .await
        .map_err(failure)?;
    if followed {
        watch::announce(&session.shared);
    }

    Ok(())
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

impl Handler for OpenNote {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: OpenNoteParams) -> Result<String, Failure> {
        let tile = store::open_note(
            pool(session).await?,
            &params.workspace,
            params.checkout.as_deref(),
            params.note.as_deref(),
        )
        .await
        .map_err(failure)?;
        watch::announce(&session.shared);

        Ok(tile)
    }
}

impl Handler for ShowNote {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: ShowNoteParams) -> Result<(), Failure> {
        store::show_note(pool(session).await?, &params.tile, params.note.as_deref())
            .await
            .map_err(failure)?;
        watch::announce(&session.shared);

        Ok(())
    }
}

impl Handler for OpenMonitor {
    const ORDERED: bool = true;

    async fn handle(session: &Session, params: OpenMonitorParams) -> Result<String, Failure> {
        let tile = store::open_monitor(
            pool(session).await?,
            &params.workspace,
            params.checkout.as_deref(),
        )
        .await
        .map_err(failure)?;
        watch::announce(&session.shared);

        Ok(tile)
    }
}

impl Handler for CloseTile {
    const ORDERED: bool = true;

    async fn handle(session: &Session, tile: String) -> Result<(), Failure> {
        store::close(pool(session).await?, &tile)
            .await
            .map_err(failure)?;
        watch::announce(&session.shared);

        Ok(())
    }
}
