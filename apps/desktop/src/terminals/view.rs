mod life;
mod size;

use std::time::Duration;

use client::{Client, State};
use gpui_kit::{App, Bounds, Context, FocusHandle, Focusable, Pixels, SharedString, Task, Window};
use protocol::terminals::Size;
use terminal::Screen;
use tokio::sync::mpsc;

use super::colors::Palette;
use super::outgoing::{self, Failure, Outgoing};
use super::route::{self, Routed};
use super::scroll::{Accumulator, Scroll};
use super::watch::Watch;
use crate::notice;
use crate::runtime;
use crate::state::store;
use crate::theme;

const SETTLE: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Opening,
    Ready,
    Ended,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub width: Pixels,
    pub height: Pixels,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stream {
    Idle,
    Starting,
    Watching,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    Background,
    Active,
    Claiming,
}

pub struct TerminalView {
    pub(super) machine: SharedString,
    pub(super) id: String,
    pub(super) client: Client,
    pub(super) screen: Screen,
    pub(super) palette: Palette,
    pub(super) status: Status,
    pub(super) connected: bool,
    pub(super) focus: FocusHandle,
    pub(super) scroll: Option<Scroll>,
    pub(super) wheel: Accumulator,
    pub(super) cell: Option<Cell>,
    pub(super) bounds: Option<Bounds<Pixels>>,
    pub(super) marked: Option<String>,
    pub(super) selecting: bool,
    pub(super) copied: Option<Task<()>>,
    pub(super) place: Place,
    pub(super) grab: Option<(Pixels, u32, Pixels)>,
    pub(super) seeking: Option<u32>,
    stream: Stream,
    watch: Option<Watch<Client>>,
    sent: Option<Size>,
    measured: Option<Size>,
    resizing: Option<Task<()>>,
    outgoing: mpsc::UnboundedSender<Outgoing>,
    _tasks: Vec<Task<()>>,
}

impl TerminalView {
    fn routed(mut events: mpsc::UnboundedReceiver<Routed>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            while let Some(first) = events.recv().await {
                let mut batch = vec![first];
                while let Ok(next) = events.try_recv() {
                    batch.push(next);
                }
                let alive = this
                    .update(cx, |this, cx| {
                        for routed in batch {
                            this.receive(routed, cx);
                        }
                        cx.notify();
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
    }

    fn receive(&mut self, routed: Routed, cx: &mut Context<Self>) {
        match routed {
            Routed::Frame(frame) => {
                if let Err(error) = self.screen.apply(&frame) {
                    tracing::warn!(%error, "could not draw a terminal frame");
                }
            }
            Routed::Scrolled(scrolled) => self.scroll = Some(Scroll::from(&scrolled)),
            Routed::Ended(ended) => {
                self.stream = Stream::Idle;
                self.status = Status::Ended;
                self.refresh(cx);
                if let Some(error) = ended.error {
                    notice::error("The terminal stopped", &error, cx);
                }
            }
        }
    }

    pub fn new(machine: SharedString, id: String, client: Client, cx: &mut Context<Self>) -> Self {
        let (outgoing, queue) = mpsc::unbounded_channel();
        let (failures, mut failed) = mpsc::channel::<Failure>(16);
        runtime::spawn(
            cx,
            outgoing::run(client.clone(), id.clone(), queue, failures),
        )
        .detach();

        let mut tasks = vec![
            cx.spawn(async move |this, cx| {
                while let Some(failure) = failed.recv().await {
                    let shown = this.update(cx, |this, cx| {
                        if this.watching() {
                            notice::error(failure.title, &failure.error, cx);
                        }
                    });
                    if shown.is_err() {
                        break;
                    }
                }
            }),
            Self::routed(route::subscribe(&machine, &client, &id, cx), cx),
        ];

        let mut states = client.watch();
        tasks.push(cx.spawn(async move |this, cx| {
            loop {
                let state = states.borrow_and_update().clone();
                if this
                    .update(cx, |this, cx| this.connection(&state, cx))
                    .is_err()
                    || states.changed().await.is_err()
                {
                    break;
                }
            }
        }));

        cx.on_release(|this, cx| this.stop(true, cx)).detach();

        Self {
            machine,
            id,
            client,
            screen: Screen::new(80, 24),
            palette: Palette::new(theme::tokens(cx)),
            status: Status::Opening,
            connected: false,
            focus: cx.focus_handle(),
            scroll: None,
            wheel: Accumulator::default(),
            cell: None,
            bounds: None,
            marked: None,
            selecting: false,
            copied: None,
            place: Place::Background,
            grab: None,
            seeking: None,
            stream: Stream::Idle,
            watch: None,
            sent: None,
            measured: None,
            resizing: None,
            outgoing,
            _tasks: tasks,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.place = match (active, self.place) {
            (false, _) => Place::Background,
            (true, Place::Claiming) => Place::Claiming,
            (true, _) => Place::Active,
        };
    }

    pub(super) fn watching(&self) -> bool {
        self.stream == Stream::Watching
    }

    pub(super) fn send(&self, outgoing: Outgoing) {
        if self.watching() {
            let _ = self.outgoing.send(outgoing);
        }
    }

    pub(super) fn send_text(&self, text: &str) {
        self.send(Outgoing::Data(text.to_owned()));
    }

    pub fn paste_text(&self, text: &str) {
        self.send_text(text);
    }

    pub(super) fn refresh(&self, cx: &mut App) {
        let machine = self.machine.clone();
        store(cx)
            .workspaces
            .clone()
            .update(cx, |workspaces, cx| workspaces.reload(&machine, cx));
    }

    fn connection(&mut self, state: &State, cx: &mut Context<Self>) {
        self.connected = *state == State::Connected;
        if self.connected {
            self.show(cx);
        } else if *state == State::Connecting {
            if self.watching() {
                self.stream = Stream::Idle;
            }
        } else {
            self.stream = Stream::Idle;
            self.stop(false, cx);
        }
        cx.notify();
    }

    pub(super) fn measured_size(&self) -> Option<Size> {
        self.measured
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
