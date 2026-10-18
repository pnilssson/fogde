use crate::Method;

/// Whether the client runs on the same computer as the host, so it can use that computer's own pickers.
pub struct IsLocal;

impl Method for IsLocal {
    const NAME: &'static str = "client.local";
    type Params = ();
    type Result = bool;
}
