use protocol::accessibility::{Accessibility, GetAccessibility, SetAccessibility};
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetAccessibility>();
    router.add::<SetAccessibility>();
}

impl Handler for GetAccessibility {
    async fn handle(session: &Session, (): ()) -> Result<Accessibility, Failure> {
        session
            .shared
            .settings
            .accessibility()
            .await
            .map_err(failure)
    }
}

impl Handler for SetAccessibility {
    async fn handle(session: &Session, accessibility: Accessibility) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_accessibility(accessibility)
            .await
            .map_err(failure)
    }
}
