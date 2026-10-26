use super::{SETTLE, Status, Stream, TerminalView};
use crate::notice;
use crate::runtime;
use crate::terminals::outgoing::Outgoing;
use crate::terminals::watch::{Queue, Watch};
use gpui_kit::Context;
use protocol::terminals::Size;

impl TerminalView {
    pub(crate) fn measured(&mut self, size: Option<Size>, cx: &mut Context<Self>) {
        if size == self.measured {
            return;
        }

        self.measured = size;
        if !self.watching() {
            self.show(cx);
            return;
        }

        self.resizing = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SETTLE).await;
            this.update(cx, |this, _| this.resize()).ok();
        }));
    }

    pub(super) fn resize(&mut self) {
        let Some(size) = self.measured else {
            return;
        };
        if !self.watching() || self.sent == Some(size) {
            return;
        }

        self.sent = Some(size);
        self.send(Outgoing::Resize(size));
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        let Some(size) = self.measured else {
            return;
        };
        if self.stream != Stream::Idle || !self.connected {
            return;
        }

        self.stream = Stream::Starting;
        self.sent = Some(size);
        let (watch, result, tail) = Watch::start(
            self.client.clone(),
            Queue::for_machine(&self.machine),
            self.id.clone(),
            size,
        );
        runtime::spawn(cx, tail).detach();
        self.watch = Some(watch);

        cx.spawn(async move |this, cx| {
            let Ok(result) = result.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.stream = Stream::Idle;
                match result {
                    Ok(()) => this.ready(cx),
                    Err(client::Error::Rpc(failure)) => {
                        this.status = Status::Failed;
                        notice::error("Could not show the terminal", &failure.message, cx);
                        this.refresh(cx);
                    }
                    Err(_) => {}
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
