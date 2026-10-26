use super::{Place, Status, Stream, TerminalView};
use crate::runtime;
use gpui_kit::{App, Context};
use std::time::Duration;

impl TerminalView {
    pub(super) fn ready(&mut self, cx: &mut Context<Self>) {
        self.stream = Stream::Watching;
        if self.place == Place::Active {
            self.place = Place::Claiming;
        }
        self.status = Status::Ready;
        self.resize();
        self.refresh(cx);
        cx.notify();
    }

    pub fn retry(&mut self, cx: &mut Context<Self>) {
        self.stop(true, cx);
        self.status = Status::Opening;
        self.show(cx);
        cx.notify();
    }

    pub(super) fn stop(&mut self, release: bool, cx: &mut App) {
        if self.watching() {
            self.stream = Stream::Idle;
        }
        if let Some(tail) = self.watch.take().and_then(|watch| watch.stop(release)) {
            runtime::spawn(cx, tail).detach();
        }
    }

    pub(crate) fn flash_copied(&mut self, cx: &mut Context<Self>) {
        self.copied = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1500))
                .await;
            this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }
}
