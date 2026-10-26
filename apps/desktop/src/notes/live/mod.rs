mod doc;
mod history;
mod input;
mod keys;
mod paint;
mod parse;
mod pointer;
mod preview;
mod view;

use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::{
    App, Context, EventEmitter, FocusHandle, Focusable, Pixels, ScrollHandle, Subscription,
    TextLayout, Window,
};

use doc::Doc;
use history::{History, Kind};
use parse::{Parsed, parse};
use preview::Line;

pub use keys::{CONTEXT, bindings};

pub enum LiveEvent {
    Changed,
    Blur,
}

pub struct Shown {
    pub line: Line,
    pub layout: TextLayout,
}

pub struct Live {
    doc: Doc,
    parsed: Parsed,
    history: History,
    marked: Option<Range<usize>>,
    goal: Option<Pixels>,
    focus: FocusHandle,
    scroll: ScrollHandle,
    shown: Rc<RefCell<Vec<Shown>>>,
    reveal: Rc<Cell<bool>>,
    dragging: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<LiveEvent> for Live {}

impl Live {
    pub fn new(text: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let subscriptions = vec![
            cx.on_focus(&focus, window, |_, _, cx| cx.notify()),
            cx.on_blur(&focus, window, |this, _, cx| {
                this.dragging = false;
                this.marked = None;
                cx.emit(LiveEvent::Blur);
                cx.notify();
            }),
        ];
        Self {
            parsed: parse(&text),
            doc: Doc::new(text),
            history: History::default(),
            marked: None,
            goal: None,
            focus,
            scroll: ScrollHandle::new(),
            shown: Rc::default(),
            reveal: Rc::default(),
            dragging: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn text(&self) -> &str {
        &self.doc.text
    }

    pub fn cursor(&self) -> usize {
        self.doc.head
    }

    pub fn apply(&mut self, range: Range<usize>, insert: &str, cx: &mut Context<Self>) {
        let (anchor, head) = (self.doc.anchor, self.doc.head);
        let moved = |offset: usize| {
            if offset <= range.start {
                offset
            } else if offset >= range.end {
                offset - range.len() + insert.len()
            } else {
                range.start + insert.len()
            }
        };
        self.doc.replace(range.clone(), insert);
        self.doc.select(moved(anchor), moved(head));
        self.history.clear();
        self.marked = None;
        self.parsed = parse(&self.doc.text);
        cx.notify();
    }

    fn set(&mut self, text: &str, cx: &mut Context<Self>) {
        self.edit(Kind::Other, cx, |doc| doc.replace(0..doc.text.len(), text));
    }

    pub fn wrap(&mut self, mark: &str, cx: &mut Context<Self>) {
        self.edit(Kind::Other, cx, |doc| doc.wrap(mark));
    }

    fn edit(&mut self, kind: Kind, cx: &mut Context<Self>, change: impl FnOnce(&mut Doc)) {
        let before = self.doc.clone();
        change(&mut self.doc);
        self.goal = None;
        self.reveal.set(true);
        if self.doc.text != before.text {
            self.history.record(&before, kind, Instant::now());
            self.parsed = parse(&self.doc.text);
            cx.emit(LiveEvent::Changed);
        }
        cx.notify();
    }

    fn restore(&mut self, doc: Doc, cx: &mut Context<Self>) {
        self.doc = doc;
        self.marked = None;
        self.goal = None;
        self.reveal.set(true);
        self.parsed = parse(&self.doc.text);
        cx.emit(LiveEvent::Changed);
        cx.notify();
    }

    fn place(&mut self, head: usize, extend: bool, cx: &mut Context<Self>) {
        self.doc.place(head, extend);
        self.marked = None;
        self.reveal.set(true);
        cx.notify();
    }

    fn line_of(&self, offset: usize) -> usize {
        self.doc.text[..offset.min(self.doc.text.len())]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
    }

    fn active(&self, window: &Window) -> Range<usize> {
        if !self.focus.is_focused(window) {
            return usize::MAX..usize::MAX;
        }

        let range = self.doc.selection();
        self.line_of(range.start)..self.line_of(range.end) + 1
    }
}

impl Focusable for Live {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
