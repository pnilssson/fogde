use protocol::rpc::Failure;
use protocol::theme::{GetMaterial, GetTheme, Material, SetMaterial, SetTheme, Theme};

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<GetTheme>();
    router.add::<SetTheme>();
    router.add::<GetMaterial>();
    router.add::<SetMaterial>();
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

impl Handler for GetMaterial {
    async fn handle(session: &Session, (): ()) -> Result<Material, Failure> {
        session.shared.settings.material().await.map_err(failure)
    }
}

impl Handler for SetMaterial {
    async fn handle(session: &Session, material: Material) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_material(material)
            .await
            .map_err(failure)
    }
}
