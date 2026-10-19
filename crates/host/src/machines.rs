use protocol::machines::{AddComputer, ListMachines, Machine, RemoveComputer};
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<ListMachines>();
    router.add::<AddComputer>();
    router.add::<RemoveComputer>();
}

impl Handler for ListMachines {
    async fn handle(session: &Session, (): ()) -> Result<Vec<Machine>, Failure> {
        let settings = &session.shared.settings;
        let (computer, servers) =
            tokio::try_join!(settings.computer(), settings.servers()).map_err(failure)?;

        Ok(computer
            .then_some(Machine::Computer)
            .into_iter()
            .chain(servers.into_iter().map(|server| Machine::Server {
                id: server.id,
                target: server.target,
            }))
            .collect())
    }
}

impl Handler for AddComputer {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_computer(true)
            .await
            .map_err(failure)
    }
}

impl Handler for RemoveComputer {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        session
            .shared
            .settings
            .set_computer(false)
            .await
            .map_err(failure)
    }
}
