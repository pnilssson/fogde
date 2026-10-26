use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Method {
    const NAME: &'static str;
    type Params: Serialize + DeserializeOwned + 'static;
    type Result: Serialize + DeserializeOwned + 'static;
}

pub trait Event {
    const NAME: &'static str;
    type Params: Serialize + DeserializeOwned + 'static;
}
