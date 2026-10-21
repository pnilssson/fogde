use protocol::notifications::{GetNotifications, Notifications, SetNotifications};
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetNotifications>();
    router.add::<SetNotifications>();
}

impl Handler for GetNotifications {
    async fn handle(session: &Session, (): ()) -> Result<Notifications, Failure> {
        session
            .shared
            .settings
            .notifications()
            .await
            .map_err(failure)
    }
}

impl Handler for SetNotifications {
    async fn handle(session: &Session, notifications: Notifications) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_notifications(notifications)
            .await
            .map_err(failure)
    }
}
