use std::marker::PhantomData;

use protocol::Event;
use serde_json::Value;
use tokio::sync::mpsc;

pub struct Listener<E> {
    receiver: mpsc::UnboundedReceiver<Value>,
    event: PhantomData<fn() -> E>,
}

impl<E: Event> Listener<E> {
    pub(crate) fn new(receiver: mpsc::UnboundedReceiver<Value>) -> Self {
        Self {
            receiver,
            event: PhantomData,
        }
    }

    pub async fn recv(&mut self) -> Option<E::Params> {
        loop {
            let value = self.receiver.recv().await?;
            if let Some(params) = parse::<E>(value) {
                return Some(params);
            }
        }
    }

    pub fn try_recv(&mut self) -> Option<E::Params> {
        loop {
            let value = self.receiver.try_recv().ok()?;
            if let Some(params) = parse::<E>(value) {
                return Some(params);
            }
        }
    }
}

fn parse<E: Event>(value: Value) -> Option<E::Params> {
    serde_json::from_value(value)
        .inspect_err(|error| tracing::warn!(%error, event = E::NAME, "could not read an event"))
        .ok()
}
