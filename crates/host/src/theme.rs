use protocol::rpc::Failure;
use protocol::theme::{GetTheme, SetTheme, Theme};

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetTheme>();
    router.add::<SetTheme>();
}

impl Handler for GetTheme {
    async fn handle(session: &Session, (): ()) -> Result<Theme, Failure> {
        session.shared.settings.theme().await.map_err(failure)
    }
}

impl Handler for SetTheme {
    async fn handle(session: &Session, theme: Theme) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_theme(theme)
            .await
            .map_err(failure)
    }
}
