use std::convert::Infallible;

use protocol::rpc::{Failure, SERVER_ERROR};
use protocol::server::{
    CheckServer, ClearServer, GetServer, ListSshHosts, ServerCheck, SetUpServer, SshKey,
};
use remote::{Check, System};
use serde_json::Value;

use super::{Session, failure, parse, reply};
use crate::settings;

impl Session {
    pub(super) async fn server(&self, params: Value) -> Result<Value, Failure> {
        parse::<GetServer>(params)?;
        let server = self.shared.settings.server().await;

        reply::<GetServer, _>(server.map(|server| server.map(|server| server.target)))
    }

    pub(super) async fn ssh_hosts(&self, params: Value) -> Result<Value, Failure> {
        parse::<ListSshHosts>(params)?;

        reply::<ListSshHosts, Infallible>(Ok(remote::hosts().await))
    }

    pub(super) async fn check_server(&self, params: Value) -> Result<Value, Failure> {
        let params = parse::<CheckServer>(params)?;
        let server = remote::Server::new(&params.target).map_err(|error| failure(&error))?;
        let check = server
            .check(params.trust.as_deref())
            .await
            .map_err(|error| failure(&error))?;

        *self.checked.lock().await = match &check {
            Check::Ready(system) => Some((server.target().to_owned(), system.clone())),
            Check::Untrusted { .. } | Check::KeyRejected { .. } => None,
        };

        reply::<CheckServer, Infallible>(Ok(match check {
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
        }))
    }

    pub(super) async fn set_up_server(&self, params: Value) -> Result<Value, Failure> {
        let server =
            remote::Server::new(&parse::<SetUpServer>(params)?).map_err(|error| failure(&error))?;
        let binaries = self.shared.binaries.as_ref().ok_or_else(|| {
            Failure::new(SERVER_ERROR, "only the desktop app can set up a server")
        })?;
        let checked = self
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
            .map_err(|error| failure(&error))?;
        self.shared
            .settings
            .set_server(Some(settings::Server {
                target: server.target().to_owned(),
                triple: system.triple,
            }))
            .await
            .map_err(|error| failure(&error))?;
        *self.shared.tunnel.lock().await = Some(tunnel);

        reply::<SetUpServer, Infallible>(Ok(()))
    }

    pub(super) async fn clear_server(&self, params: Value) -> Result<Value, Failure> {
        parse::<ClearServer>(params)?;
        self.shared.tunnel.lock().await.take();

        reply::<ClearServer, _>(self.shared.settings.set_server(None).await)
    }
}

async fn ready(server: &remote::Server) -> Result<System, Failure> {
    match server.check(None).await.map_err(|error| failure(&error))? {
        Check::Ready(system) => Ok(system),
        Check::Untrusted { .. } => Err(Failure::new(
            SERVER_ERROR,
            format!("{} is not trusted yet", server.target()),
        )),
        Check::KeyRejected { .. } => Err(failure(&remote::Error::Key(server.target().to_owned()))),
    }
}
