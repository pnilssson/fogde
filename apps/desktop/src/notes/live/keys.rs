mod bind;

pub use bind::bindings;

use gpui_kit::component::input::{Copy, Cut, Paste, Redo, SelectAll, Undo};
use gpui_kit::{ClipboardItem, Context, actions};

use super::Live;
use super::doc::{line_end, line_start, next, previous, word_left, word_right};
use super::history::Kind;

actions!(
    note,
    [
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        WordLeft,
        WordRight,
        SelectWordLeft,
        SelectWordRight,
        Home,
        End,
        SelectHome,
        SelectEnd,
        Top,
        Bottom,
        SelectTop,
        SelectBottom,
        PageUp,
        PageDown,
        Backspace,
        Delete,
        DeleteWordLeft,
        DeleteWordRight,
        DeleteHome,
        Enter,
        Indent,
        Outdent,
    ]
);

pub const CONTEXT: &str = "Note";

impl Live {
    fn horizontal(&mut self, forward: bool, extend: bool, cx: &mut Context<Self>) {
        let text = &self.doc.text;
        let range = self.doc.selection();
        let head = match (extend, range.is_empty(), forward) {
            (false, false, false) => range.start,
            (false, false, true) => range.end,
            (_, _, false) => previous(text, self.doc.head),
            (_, _, true) => next(text, self.doc.head),
        };
        self.goal = None;
        self.place(head, extend, cx);
    }

    fn jump(&mut self, head: usize, extend: bool, cx: &mut Context<Self>) {
        self.goal = None;
        self.place(head, extend, cx);
    }

    fn copy(&self, cx: &mut Context<Self>) {
        if !self.doc.collapsed() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.doc.selected().to_owned()));
        }
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = text.replace("\r\n", "\n");
        self.edit(Kind::Other, cx, |doc| doc.insert(&text));
    }

    fn undo(&mut self, redo: bool, cx: &mut Context<Self>) {
        let restored = if redo {
            self.history.redo(&self.doc)
        } else {
            self.history.undo(&self.doc)
        };
        if let Some(doc) = restored {
            self.restore(doc, cx);
        }
    }

    fn motion(&self, any: &dyn std::any::Any) -> Option<(usize, bool)> {
        let (text, head) = (&self.doc.text, self.doc.head);
        let found = [
            (any.is::<WordLeft>(), word_left(text, head), false),
            (any.is::<SelectWordLeft>(), word_left(text, head), true),
            (any.is::<WordRight>(), word_right(text, head), false),
            (any.is::<SelectWordRight>(), word_right(text, head), true),
            (any.is::<Home>(), line_start(text, head), false),
            (any.is::<SelectHome>(), line_start(text, head), true),
            (any.is::<End>(), line_end(text, head), false),
            (any.is::<SelectEnd>(), line_end(text, head), true),
            (any.is::<Top>(), 0, false),
            (any.is::<SelectTop>(), 0, true),
            (any.is::<Bottom>(), text.len(), false),
            (any.is::<SelectBottom>(), text.len(), true),
        ];
        found
            .into_iter()
            .find(|(matched, _, _)| *matched)
            .map(|(_, target, extend)| (target, extend))
    }

    pub(super) fn command(&mut self, action: &dyn gpui_kit::Action, cx: &mut Context<Self>) {
        let any = action.as_any();
        if let Some((target, extend)) = self.motion(any) {
            self.jump(target, extend, cx);
            return;
        }

        let (text, head) = (self.doc.text.clone(), self.doc.head);
        if any.is::<Backspace>() {
            self.edit(Kind::Typing, cx, super::doc::Doc::delete_backward);
        } else if any.is::<Delete>() {
            self.edit(Kind::Typing, cx, super::doc::Doc::delete_forward);
        } else if any.is::<DeleteWordLeft>() {
            self.edit(Kind::Other, cx, |doc| doc.delete_to(word_left(&text, head)));
        } else if any.is::<DeleteWordRight>() {
            self.edit(Kind::Other, cx, |doc| {
                doc.delete_to(word_right(&text, head));
            });
        } else if any.is::<DeleteHome>() {
            self.edit(Kind::Other, cx, |doc| {
                doc.delete_to(line_start(&text, head));
            });
        } else if any.is::<Enter>() {
            self.edit(Kind::Other, cx, super::doc::Doc::newline);
        } else if any.is::<Indent>() {
            self.edit(Kind::Other, cx, super::doc::Doc::indent);
        } else if any.is::<Outdent>() {
            self.edit(Kind::Other, cx, super::doc::Doc::outdent);
        } else if any.is::<SelectAll>() {
            self.doc.select(0, text.len());
            cx.notify();
        } else if any.is::<Copy>() {
            self.copy(cx);
        } else if any.is::<Cut>() {
            self.copy(cx);
            self.edit(Kind::Other, cx, |doc| doc.insert(""));
        } else if any.is::<Paste>() {
            self.paste(cx);
        } else if any.is::<Undo>() || any.is::<Redo>() {
            self.undo(any.is::<Redo>(), cx);
        } else if any.is::<Left>() || any.is::<SelectLeft>() {
            self.horizontal(false, any.is::<SelectLeft>(), cx);
        } else if any.is::<Right>() || any.is::<SelectRight>() {
            self.horizontal(true, any.is::<SelectRight>(), cx);
        } else if any.is::<Up>() || any.is::<SelectUp>() {
            self.vertical(-1., any.is::<SelectUp>(), cx);
        } else if any.is::<Down>() || any.is::<SelectDown>() {
            self.vertical(1., any.is::<SelectDown>(), cx);
        } else if any.is::<PageUp>() {
            self.vertical(-self.page(), false, cx);
        } else if any.is::<PageDown>() {
            self.vertical(self.page(), false, cx);
        }
    }
}

macro_rules! listen {
    ($element:expr, $cx:expr, [$($action:ty),* $(,)?]) => {
        $element$(.on_action($cx.listener(|this: &mut Live, action: &$action, _, cx| this.command(action, cx))))*
    };
}

pub(super) fn listen(
    element: gpui_kit::Stateful<gpui_kit::Div>,
    cx: &mut Context<Live>,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    use gpui_kit::InteractiveElement;

    listen!(
        element,
        cx,
        [
            Left,
            Right,
            Up,
            Down,
            SelectLeft,
            SelectRight,
            SelectUp,
            SelectDown,
            WordLeft,
            WordRight,
            SelectWordLeft,
            SelectWordRight,
            Home,
            End,
            SelectHome,
            SelectEnd,
            Top,
            Bottom,
            SelectTop,
            SelectBottom,
            PageUp,
            PageDown,
            Backspace,
            Delete,
            DeleteWordLeft,
            DeleteWordRight,
            DeleteHome,
            Enter,
            Indent,
            Outdent,
            SelectAll,
            Copy,
            Cut,
            Paste,
            Undo,
            Redo,
        ]
    )
}
