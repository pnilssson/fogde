mod channel;
mod client;
mod error;
mod keepalive;
mod listener;
mod server;

pub use client::{Client, State};
pub use error::Error;
pub use listener::Listener;

use std::sync::{Mutex, MutexGuard, PoisonError};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
