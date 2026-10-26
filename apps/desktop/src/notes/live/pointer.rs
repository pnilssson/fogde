use gpui_kit::{
    Bounds, Context, MouseDownEvent, MouseMoveEvent, Pixels, Point, Window, point, px, size,
};

use super::Live;
use super::doc::{line_end, line_start, word_left, word_right};
use super::preview::Widget;

impl Live {
    fn identity(shown: &super::Shown) -> bool {
        shown.line.widget == Widget::None
            && shown.line.map.len() == 1
            && shown.line.map[0].1 == shown.line.range.start
    }

    pub(super) fn caret_at(&self, offset: usize) -> Option<(Point<Pixels>, Pixels)> {
        let shown = self.shown.borrow();
        let found = shown
            .iter()
            .find(|shown| shown.line.range.start <= offset && offset <= shown.line.range.end)?;
        let index = if Self::identity(found) {
            offset - found.line.range.start
        } else {
            found.line.shown(offset).unwrap_or(0)
        };
        let height = found.layout.line_height();
        let position = found
            .layout
            .position_for_index(index)
            .unwrap_or(found.layout.bounds().origin);
        Some((position, height))
    }

    pub(super) fn caret_bounds(&self, offset: usize) -> Option<Bounds<Pixels>> {
        let (position, height) = self.caret_at(offset)?;
        Some(Bounds::new(position, size(px(2.), height)))
    }

    pub(super) fn offset_at(&self, at: Point<Pixels>) -> usize {
        let shown = self.shown.borrow();
        let Some(last) = shown.last() else {
            return 0;
        };
        if at.y > last.layout.bounds().bottom() {
            return self.doc.text.len();
        }

        let found = shown
            .iter()
            .find(|shown| at.y <= shown.layout.bounds().bottom())
            .unwrap_or(last);
        let bounds = found.layout.bounds();
        let inside = point(at.x, at.y.clamp(bounds.top(), bounds.bottom() - px(1.)));
        let index = match found.layout.index_for_position(inside) {
            Ok(index) | Err(index) => index.min(found.line.text.len()),
        };
        found.line.source(index)
    }

    pub(super) fn vertical(&mut self, rows: f32, extend: bool, cx: &mut Context<Self>) {
        let Some((position, height)) = self.caret_at(self.doc.head) else {
            return;
        };
        let x = *self.goal.get_or_insert(position.x);
        let target = point(x, position.y + height * rows + height / 2.);
        let top = self
            .shown
            .borrow()
            .first()
            .map(|shown| shown.layout.bounds().top());
        let head = if rows < 0. && top.is_some_and(|top| target.y < top) {
            0
        } else {
            self.offset_at(target)
        };
        let goal = self.goal;
        self.place(head, extend, cx);
        self.goal = goal;
    }

    pub(super) fn page(&self) -> f32 {
        let viewport = self.scroll.bounds().size.height;
        let line = self
            .shown
            .borrow()
            .first()
            .map_or(px(20.), |shown| shown.layout.line_height());
        (viewport / line).floor().max(1.)
    }

    pub(super) fn mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus.focus(window, cx);
        let offset = self.offset_at(event.position);
        let text = &self.doc.text;
        match event.click_count {
            2 => {
                let end = word_right(text, offset);
                self.doc.select(word_left(text, end), end);
            }
            3.. => {
                let end = line_end(text, offset);
                let end = if end < text.len() { end + 1 } else { end };
                self.doc.select(line_start(text, offset), end);
            }
            _ => self.doc.place(offset, event.modifiers.shift),
        }
        self.goal = None;
        self.marked = None;
        self.dragging = true;
        cx.notify();
    }

    pub(super) fn mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.dragging || !event.dragging() {
            return;
        }

        let offset = self.offset_at(event.position);
        self.place(offset, true, cx);
    }

    pub(super) fn mouse_up(&mut self) {
        self.dragging = false;
    }

    pub(super) fn toggle(&mut self, at: usize, cx: &mut Context<Self>) {
        self.edit(super::history::Kind::Other, cx, |doc| doc.toggle_task(at));
    }
}
