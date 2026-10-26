mod edit;

use std::rc::Rc;

use futures_util::FutureExt;
use futures_util::future::Shared;
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Styled, Subscription, Task, Window, actions, div,
};
use protocol::notes::{NoteText, WriteNote, WriteNoteParams, Written};

use super::live::{Live, LiveEvent};
use super::session::{Outcome, PAUSE, Session};
use crate::notice;
use crate::runtime;
use crate::state::machines::Machine;

actions!(notes, [Bold, Italic]);

pub const CONTEXT: &str = "NoteEditor";

pub type Follow = Rc<dyn Fn(usize, &str, &mut Window, &mut App)>;

type Write = Shared<Task<Result<Written, client::Error>>>;

pub struct NoteEditor {
    machine: Machine,
    path: String,
    state: Entity<Live>,
    session: Session,
    crlf: bool,
    pending: Option<Task<()>>,
    writing: Option<(String, Write)>,
    follow: Follow,
    _subscriptions: Vec<Subscription>,
}

fn display(text: &str) -> String {
    text.replace("\r\n", "\n")
}

impl NoteEditor {
    pub fn new(
        machine: Machine,
        path: String,
        loaded: NoteText,
        follow: Follow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let shown = display(&loaded.text);
        let empty = shown.is_empty();
        let state = cx.new(|cx| Live::new(shown, window, cx));
        let subscriptions = vec![cx.subscribe_in(
            &state,
            window,
            |this: &mut Self, _, event: &LiveEvent, window, cx| match event {
                LiveEvent::Changed => this.schedule(window, cx),
                LiveEvent::Blur => this.flush(window, cx),
            },
        )];
        if empty {
            let focus = state.focus_handle(cx);
            window.focus(&focus, cx);
        }
        cx.on_release(|this: &mut Self, cx| this.flush_detached(cx))
            .detach();

        Self {
            machine,
            path,
            state,
            session: Session::new(loaded.text.clone(), loaded.version),
            crlf: loaded.text.contains("\r\n"),
            pending: None,
            writing: None,
            follow,
            _subscriptions: subscriptions,
        }
    }

    fn disk(&self, cx: &App) -> String {
        let shown = self.state.read(cx).text().to_owned();
        if self.crlf {
            shown.replace('\n', "\r\n")
        } else {
            shown
        }
    }

    fn schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(PAUSE).await;
            this.update_in(cx, Self::save).ok();
        }));
    }

    pub fn flush(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending = None;
        self.save(window, cx);
    }

    fn flush_detached(&mut self, cx: &mut App) {
        let text = self.disk(cx);
        if let Some((sent, write)) = self.writing.take() {
            if text != sent {
                self.write_after(write, text, cx);
            }
            return;
        }

        let Some((text, version)) = self.session.begin(&text) else {
            return;
        };
        let params = WriteNoteParams {
            path: self.path.clone(),
            text,
            version,
        };
        runtime::call::<WriteNote>(&self.machine.client, params, cx).detach();
    }

    fn write_after(&self, write: Write, text: String, cx: &mut App) {
        let client = self.machine.client.clone();
        let path = self.path.clone();
        cx.spawn(async move |cx| {
            let Ok(Written::Saved { version }) = write.await else {
                return;
            };

            let params = WriteNoteParams {
                path,
                text,
                version,
            };
            cx.update(|cx| runtime::call::<WriteNote>(&client, params, cx).detach());
        })
        .detach();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.disk(cx);
        let Some((text, version)) = self.session.begin(&text) else {
            return;
        };
        let params = WriteNoteParams {
            path: self.path.clone(),
            text: text.clone(),
            version,
        };
        let write = runtime::call::<WriteNote>(&self.machine.client, params, cx).shared();
        self.writing = Some((text.clone(), write.clone()));
        cx.spawn_in(window, async move |this, cx| {
            let result = write.await;
            this.update_in(cx, |this, window, cx| {
                this.writing = None;
                let now = this.disk(cx);
                let outcome =
                    result.map_err(|error| notice::error("Could not save the note", &error, cx));
                let (saved, waiting) = this.session.finish(text, outcome, &now);
                if let Some((text, version)) = waiting {
                    this.apply(text, version, window, cx);
                    return;
                }
                match saved {
                    Outcome::Saved { again: true, .. } => this.schedule(window, cx),
                    Outcome::Changed => this.reread(window, cx),
                    Outcome::Saved { .. } | Outcome::Failed => {}
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn reread(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let request =
            runtime::call::<protocol::notes::ReadNote>(&self.machine.client, self.path.clone(), cx);
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(loaded) = request.await {
                this.update_in(cx, |this, window, cx| {
                    this.apply(loaded.text, loaded.version, window, cx);
                })
                .ok();
            }
        })
        .detach();
    }
}

impl Render for NoteEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(CONTEXT)
            .size_full()
            .on_action(cx.listener(|this, _: &Bold, _, cx| this.wrap("**", cx)))
            .on_action(cx.listener(|this, _: &Italic, _, cx| this.wrap("*", cx)))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &gpui_kit::MouseUpEvent, window, cx| {
                    if event.modifiers.secondary(cx) {
                        this.click(window, cx);
                    }
                }),
            )
            .child(self.state.clone())
    }
}

impl Focusable for NoteEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}
