use crate::Method;

pub struct IsLocal;

impl Method for IsLocal {
    const NAME: &'static str = "client.local";
    type Params = ();
    type Result = bool;
}
