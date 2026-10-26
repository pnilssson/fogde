use std::future::Future;

use client::Client;
use gpui_kit::{App, AppContext, Global, Task};
use protocol::Method;
use tokio::runtime::Handle;
use tokio::task::AbortHandle;

struct Runtime(Handle);

impl Global for Runtime {}

struct Abort(AbortHandle);

impl Drop for Abort {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub fn init(handle: Handle, cx: &mut App) {
    cx.set_global(Runtime(handle));
}

pub fn handle(cx: &App) -> Handle {
    cx.global::<Runtime>().0.clone()
}

pub fn spawn<F>(cx: &App, future: F) -> Task<Option<F::Output>>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let task = cx.global::<Runtime>().0.spawn(future);
    let abort = Abort(task.abort_handle());
    cx.background_spawn(async move {
        let output = task.await.ok();
        drop(abort);
        output
    })
}

pub fn call<M>(
    client: &Client,
    params: M::Params,
    cx: &App,
) -> Task<Result<M::Result, client::Error>>
where
    M: Method,
    M::Params: Send + Sync,
    M::Result: Send,
{
    let client = client.clone();
    let task = spawn(cx, async move { client.call::<M>(params).await });
    cx.background_spawn(async move { task.await.unwrap_or(Err(client::Error::Closed)) })
}
