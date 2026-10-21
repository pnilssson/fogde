mod store;

use protocol::machines::{
    AddComputer, ConnectMachine, DisconnectMachine, ListMachines, Machine, MachineId,
    RemoveMachine, RenameMachine, RenameMachineParams,
};
use protocol::rpc::{Failure, SERVER_ERROR};
use sqlx::SqlitePool;

use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
use crate::shared::Shared;

pub use store::Server;

pub fn register(router: &mut Router) {
    router.add::<ListMachines>();
    router.add::<AddComputer>();
    router.add::<ConnectMachine>();
    router.add::<DisconnectMachine>();
    router.add::<RenameMachine>();
    router.add::<RemoveMachine>();
}

const COMPUTER: &str = "This computer";

pub async fn add_server(shared: &Shared, server: &Server) -> Result<(), Failure> {
    store::add_server(pool(shared).await?, server)
        .await
        .map_err(failure)
}

pub async fn server(shared: &Shared, id: &str) -> Result<Server, Failure> {
    let machine = store::server(pool(shared).await?, id)
        .await
        .map_err(failure)?;

    match machine {
        Some(store::Machine {
            server: Some(server),
            connected: true,
            ..
        }) => Ok(server),
        Some(_) => Err(Failure::new(SERVER_ERROR, "that server is disconnected")),
        None => Err(Failure::new(
            SERVER_ERROR,
            "that server is no longer set up",
        )),
    }
}

impl Handler for ListMachines {
    async fn handle(session: &Session, (): ()) -> Result<Vec<Machine>, Failure> {
        let machines = store::list(pool(&session.shared).await?)
            .await
            .map_err(failure)?;

        Ok(machines
            .into_iter()
            .map(|machine| match machine.server {
                None => Machine::Computer {
                    name: machine.name.unwrap_or_else(|| COMPUTER.to_owned()),
                    connected: machine.connected,
                },
                Some(server) => Machine::Server {
                    id: server.id,
                    name: machine.name.unwrap_or_else(|| server.target.clone()),
                    target: server.target,
                    connected: machine.connected,
                },
            })
            .collect())
    }
}

impl Handler for AddComputer {
    async fn handle(session: &Session, (): ()) -> Result<(), Failure> {
        store::add_computer(pool(&session.shared).await?)
            .await
            .map_err(failure)
    }
}

impl Handler for ConnectMachine {
    async fn handle(session: &Session, machine: MachineId) -> Result<(), Failure> {
        store::set_connected(pool(&session.shared).await?, &machine, true)
            .await
            .map_err(failure)
    }
}

impl Handler for DisconnectMachine {
    async fn handle(session: &Session, machine: MachineId) -> Result<(), Failure> {
        store::set_connected(pool(&session.shared).await?, &machine, false)
            .await
            .map_err(failure)?;
        close(&session.shared, &machine).await;

        Ok(())
    }
}

impl Handler for RenameMachine {
    async fn handle(session: &Session, params: RenameMachineParams) -> Result<(), Failure> {
        store::rename(pool(&session.shared).await?, &params.machine, &params.name)
            .await
            .map_err(failure)
    }
}

impl Handler for RemoveMachine {
    async fn handle(session: &Session, machine: MachineId) -> Result<(), Failure> {
        store::remove(pool(&session.shared).await?, &machine)
            .await
            .map_err(failure)?;
        close(&session.shared, &machine).await;

        Ok(())
    }
}

async fn close(shared: &Shared, machine: &MachineId) {
    if let MachineId::Server { id } = machine {
        shared.tunnels.lock().await.remove(id);
    }
}

async fn pool(shared: &Shared) -> Result<&SqlitePool, Failure> {
    shared.db.pool().await.map_err(failure)
}
