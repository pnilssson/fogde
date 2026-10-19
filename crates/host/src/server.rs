use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::server::{
    CheckServer, ConnectServer, ListSshHosts, RemoveServer, ServerCheck, ServerCheckParams,
    SetUpServer, SshKey,
};
use remote::{Check, System};
use tokio::sync::Mutex;

use crate::relay;
use crate::rpc::{Handler, Router, failure};
use crate::session::Session;
use crate::settings;

#[derive(Default)]
pub struct State {
    checked: Mutex<Option<(String, System)>>,
}

pub fn register(router: &mut Router) {
    router.add::<ListSshHosts>();
    router.add::<CheckServer>();
    router.add::<SetUpServer>();
    router.add::<ConnectServer>();
    router.add::<RemoveServer>();
}

impl Handler for ListSshHosts {
    async fn handle(_: &Session, (): ()) -> Result<Vec<String>, Failure> {
        Ok(remote::hosts().await)
    }
}

impl Handler for CheckServer {
    async fn handle(session: &Session, params: ServerCheckParams) -> Result<ServerCheck, Failure> {
        let server = remote::Server::new(&params.target).map_err(failure)?;
        let check = server
            .check(params.trust.as_deref())
            .await
            .map_err(failure)?;

        *session.server.checked.lock().await = match &check {
            Check::Ready(system) => Some((server.target().to_owned(), system.clone())),
            Check::Untrusted { .. } | Check::KeyRejected { .. } => None,
        };

        Ok(match check {
            Check::Ready(_) => ServerCheck::Ready,
            Check::Untrusted { fingerprint } => ServerCheck::Untrusted { fingerprint },
            Check::KeyRejected { keys } => ServerCheck::KeyRejected {
                keys: keys
                    .into_iter()
                    .map(|key| SshKey {
                        name: key.name,
                        fingerprint: key.fingerprint,
                    })
                    .collect(),
            },
        })
    }
}

impl Handler for SetUpServer {
    async fn handle(session: &Session, target: String) -> Result<String, Failure> {
        let server = remote::Server::new(&target).map_err(failure)?;
        let binaries = session.shared.binaries.as_ref().ok_or_else(|| {
            Failure::new(SERVER_ERROR, "only the desktop app can set up a server")
        })?;
        let checked = session
            .server
            .checked
            .lock()
            .await
            .take()
            .filter(|(target, _)| target == server.target());
        let system = match checked {
            Some((_, system)) => system,
            None => ready(&server).await?,
        };

        let tunnel = server
            .open(&system, binaries)
            .await
            .map_err(relay::unreachable)?;
        let id = tunnel.host.clone();
        session
            .shared
            .settings
            .add_server(settings::Server {
                id: id.clone(),
                target: server.target().to_owned(),
                triple: system.triple,
            })
            .await
            .map_err(failure)?;
        session
            .shared
            .tunnels
            .lock()
            .await
            .insert(id.clone(), tunnel);

        Ok(id)
    }
}

impl Handler for ConnectServer {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        relay::connect(&session.shared, &id).await
    }
}

impl Handler for RemoveServer {
    async fn handle(session: &Session, id: String) -> Result<(), Failure> {
        session.shared.tunnels.lock().await.remove(&id);

        session
            .shared
            .settings
            .remove_server(&id)
            .await
            .map_err(failure)
    }
}

async fn ready(server: &remote::Server) -> Result<System, Failure> {
    match server.check(None).await.map_err(failure)? {
        Check::Ready(system) => Ok(system),
        Check::Untrusted { .. } => Err(Failure::new(
            SERVER_ERROR,
            format!("{} is not trusted yet", server.target()),
        )),
        Check::KeyRejected { .. } => Err(failure(remote::Error::Key(server.target().to_owned()))),
    }
}
