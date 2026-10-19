use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Method;

pub struct ListSshHosts;

impl Method for ListSshHosts {
    const NAME: &'static str = "server.hosts";
    type Params = ();
    type Result = Vec<String>;
}

#[derive(Clone, Debug, Deserialize, TS)]
pub struct ServerCheckParams {
    pub target: String,
    pub trust: Option<String>,
}

#[derive(Clone, Debug, Serialize, TS)]
pub struct SshKey {
    pub name: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum ServerCheck {
    Ready,
    Untrusted { fingerprint: String },
    KeyRejected { keys: Vec<SshKey> },
}

pub struct CheckServer;

impl Method for CheckServer {
    const NAME: &'static str = "server.check";
    type Params = ServerCheckParams;
    type Result = ServerCheck;
}

pub struct SetUpServer;

impl Method for SetUpServer {
    const NAME: &'static str = "server.setUp";
    type Params = String;
    type Result = String;
}

pub struct ConnectServer;

impl Method for ConnectServer {
    const NAME: &'static str = "server.connect";
    type Params = String;
    type Result = ();
}

pub struct RemoveServer;

impl Method for RemoveServer {
    const NAME: &'static str = "server.remove";
    type Params = String;
    type Result = ();
}
