use crate::Method;

pub struct Ping;

impl Method for Ping {
    const NAME: &'static str = "health.ping";
    type Params = ();
    type Result = ();
}
