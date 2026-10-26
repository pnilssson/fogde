use client::{Client, Listener};
use gpui_kit::{Context, Task};
use protocol::Event;

pub fn drain<E, T>(
    client: &Client,
    cx: &mut Context<T>,
    handle: impl Fn(&mut T, E::Params, &mut Context<T>) -> bool + 'static,
) -> Task<()>
where
    E: Event + 'static,
    T: 'static,
{
    let mut listener: Listener<E> = client.listen::<E>();
    cx.spawn(async move |this, cx| {
        while let Some(first) = listener.recv().await {
            let mut batch = vec![first];
            while let Some(next) = listener.try_recv() {
                batch.push(next);
            }

            let alive = this
                .update(cx, |this, cx| {
                    let mut changed = false;
                    for params in batch {
                        changed |= handle(this, params, cx);
                    }
                    if changed {
                        cx.notify();
                    }
                })
                .is_ok();
            if !alive {
                break;
            }
        }
    })
}
