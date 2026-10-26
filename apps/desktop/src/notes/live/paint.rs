use std::ops::Range;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Bounds, Context, ElementInputHandler, IntoElement, Pixels, Styled, TextLayout, canvas, fill,
    point, px,
};

use super::{Live, Shown};
use crate::theme;

const MARGIN: Pixels = px(24.);

fn rects(layout: &TextLayout, range: &Range<usize>, newline: bool) -> Vec<Bounds<Pixels>> {
    let (Some(start), Some(end)) = (
        layout.position_for_index(range.start),
        layout.position_for_index(range.end),
    ) else {
        return Vec::new();
    };
    let height = layout.line_height();
    let bounds = layout.bounds();
    let tail = if newline { px(4.) } else { px(0.) };
    if start.y == end.y {
        return vec![Bounds::from_corners(
            start,
            point(end.x + tail, end.y + height),
        )];
    }

    let mut found = vec![Bounds::from_corners(
        start,
        point(bounds.right(), start.y + height),
    )];
    if end.y > start.y + height {
        found.push(Bounds::from_corners(
            point(bounds.left(), start.y + height),
            point(bounds.right(), end.y),
        ));
    }
    found.push(Bounds::from_corners(
        point(bounds.left(), end.y),
        point(end.x + tail, end.y + height),
    ));
    found
}

fn selection(shown: &[Shown], range: &Range<usize>) -> Vec<Bounds<Pixels>> {
    shown
        .iter()
        .filter(|shown| shown.line.map.len() == 1 && shown.line.map[0].1 == shown.line.range.start)
        .filter_map(|shown| {
            let line = &shown.line.range;
            let start = range.start.max(line.start);
            let end = range.end.min(line.end);
            let crosses = range.end > line.end && range.start <= line.end;
            (start < end || crosses).then(|| {
                rects(
                    &shown.layout,
                    &(start - line.start..end.max(start) - line.start),
                    crosses,
                )
            })
        })
        .flatten()
        .collect()
}

impl Live {
    pub(super) fn background(&self, focused: bool, cx: &Context<Self>) -> impl IntoElement {
        let shown = self.shown.clone();
        let range = self.doc.selection();
        let color = theme::tokens(cx).selected;
        canvas(
            |_, _, _| {},
            move |_, (), window, _| {
                if !focused || range.is_empty() {
                    return;
                }
                for rect in selection(&shown.borrow(), &range) {
                    window.paint_quad(fill(rect, color));
                }
            },
        )
        .absolute()
        .size_full()
    }

    pub(super) fn foreground(&self, focused: bool, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let focus = self.focus.clone();
        let (head, marked) = (self.doc.head, self.marked.clone());
        let (reveal, scroll) = (self.reveal.clone(), self.scroll.clone());
        let caret = cx.theme().foreground;
        canvas(
            |_, _, _| {},
            move |bounds, (), window, cx| {
                let live = entity.read(cx);
                let cursor = focused.then(|| live.caret_bounds(head)).flatten();
                let underline = marked.as_ref().and_then(|marked| {
                    Some((live.caret_at(marked.start)?, live.caret_at(marked.end)?))
                });
                if let Some(cursor) = cursor {
                    window.paint_quad(fill(cursor, caret));
                }
                if let Some(((from, height), (to, _))) =
                    underline.filter(|((from, _), (to, _))| from.y == to.y)
                {
                    let mark = Bounds::from_corners(
                        point(from.x, from.y + height - px(2.)),
                        point(to.x, from.y + height - px(1.)),
                    );
                    window.paint_quad(fill(mark, caret));
                }
                window.handle_input(&focus, ElementInputHandler::new(bounds, entity.clone()), cx);

                let Some(cursor) = cursor.filter(|_| reveal.replace(false)) else {
                    return;
                };
                let viewport = scroll.bounds();
                let mut offset = scroll.offset();
                if cursor.top() < viewport.top() + MARGIN {
                    offset.y += viewport.top() + MARGIN - cursor.top();
                } else if cursor.bottom() > viewport.bottom() - MARGIN {
                    offset.y -= cursor.bottom() - (viewport.bottom() - MARGIN);
                } else {
                    return;
                }
                offset.y = offset.y.min(px(0.)).max(-scroll.max_offset().y);
                scroll.set_offset(offset);
                window.refresh();
            },
        )
        .absolute()
        .size_full()
    }
}
