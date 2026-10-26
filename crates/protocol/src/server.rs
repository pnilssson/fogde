use serde::{Deserialize, Serialize};

use crate::Method;

pub struct ListSshHosts;

impl Method for ListSshHosts {
    const NAME: &'static str = "server.hosts";
    type Params = ();
    type Result = Vec<String>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerCheckParams {
    pub target: String,
    pub trust: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SshKey {
    pub name: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
