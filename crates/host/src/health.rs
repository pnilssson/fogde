use std::future;

use protocol::health::Ping;
use protocol::rpc::Failure;

use crate::rpc::{Handler, Router};
use crate::session::Session;

pub fn register(router: &mut Router) {
    router.add::<Ping>();
}

impl Handler for Ping {
    fn handle(_: &Session, (): ()) -> impl Future<Output = Result<(), Failure>> + Send {
        future::ready(Ok(()))
    }
}
