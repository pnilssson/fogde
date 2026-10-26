use std::ops::Range;

use gpui_kit::{
    Bounds, Context, EntityInputHandler, KeyDownEvent, Pixels, Point, UTF16Selection, Window, point,
};

use super::keys::{Press, press};
use super::outgoing::Outgoing;
use super::select;
use super::view::TerminalView;

impl TerminalView {
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.marked.is_some() {
            return;
        }

        let macos = cfg!(target_os = "macos");
        let selection = select::selected(self).is_some();
        match press(&event.keystroke, macos, selection) {
            Press::Text | Press::App => return,
            Press::Bytes(bytes) => {
                select::clear(self);
                self.send(Outgoing::Data(String::from_utf8_lossy(&bytes).into_owned()));
            }
            Press::Named(key) => self.send(Outgoing::Key(key)),
            Press::Copy => {
                select::copy(self, cx);
                select::clear(self);
            }
            Press::Paste => select::paste(self, cx),
        }
        window.prevent_default();
        cx.stop_propagation();
        cx.notify();
    }

    fn cursor_bounds(&self) -> Option<Bounds<Pixels>> {
        let (bounds, cell) = (self.bounds?, self.cell?);
        let cursor = self.screen.term().grid().cursor.point;
        let row = u16::try_from(cursor.line.0).ok()?;
        let column = u16::try_from(cursor.column.0).ok()?;
        Some(Bounds::new(
            point(
                bounds.origin.x + cell.width * f32::from(column),
                bounds.origin.y + cell.height * f32::from(row),
            ),
            gpui_kit::size(cell.width, cell.height),
        ))
    }
}

fn utf16(text: &str) -> usize {
    text.encode_utf16().count()
}

impl EntityInputHandler for TerminalView {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let marked = self.marked.as_ref()?;
        let units: Vec<u16> = marked.encode_utf16().collect();
        let end = range.end.min(units.len());
        let start = range.start.min(end);
        *adjusted = Some(start..end);
        String::from_utf16(&units[start..end]).ok()
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let at = self.marked.as_deref().map_or(0, utf16);
        Some(UTF16Selection {
            range: at..at,
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_deref().map(|text| 0..utf16(text))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.marked = None;
        if !text.is_empty() {
            select::clear(self);
            self.send_text(text);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        text: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.marked = (!text.is_empty()).then(|| text.to_owned());
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        self.cursor_bounds()
    }

    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.watching()
    }
}
