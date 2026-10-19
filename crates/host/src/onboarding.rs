use protocol::onboarding::{FinishOnboarding, GetOnboarded};
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetOnboarded>();
    router.add::<FinishOnboarding>();
}

impl Handler for GetOnboarded {
    async fn handle(session: &Session, (): ()) -> Result<bool, Failure> {
        session.shared.settings.onboarded().await.map_err(failure)
    }
}

impl Handler for FinishOnboarding {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        session
            .shared
            .settings
            .finish_onboarding()
            .await
            .map_err(failure)
    }
}
