use protocol::rpc::Failure;
use protocol::stats::{GetStats, SetStats, Stats};

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetStats>();
    router.add::<SetStats>();
}

impl Handler for GetStats {
    async fn handle(session: &Session, (): ()) -> Result<Stats, Failure> {
        session.shared.settings.stats().await.map_err(failure)
    }
}

impl Handler for SetStats {
    async fn handle(session: &Session, stats: Stats) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_stats(stats)
            .await
            .map_err(failure)
    }
}
