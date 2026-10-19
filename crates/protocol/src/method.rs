use serde::Serialize;
use serde::de::DeserializeOwned;
use ts_rs::TS;

pub trait Method {
    const NAME: &'static str;
    type Params: DeserializeOwned + TS + 'static;
    type Result: Serialize + TS + 'static;
}

pub trait Event {
    const NAME: &'static str;
    type Params: Serialize + TS + 'static;
}
