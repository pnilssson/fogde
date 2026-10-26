use std::ops::Range;

use gpui_kit::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window};

use super::Live;
use super::history::Kind;

fn to_utf16(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].encode_utf16().count()
}

fn from_utf16(text: &str, units: usize) -> usize {
    let mut count = 0;
    for (index, char) in text.char_indices() {
        if count >= units {
            return index;
        }
        count += char.len_utf16();
    }
    text.len()
}

impl Live {
    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        to_utf16(&self.doc.text, range.start)..to_utf16(&self.doc.text, range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        from_utf16(&self.doc.text, range.start)..from_utf16(&self.doc.text, range.end)
    }

    fn target(&self, range: Option<Range<usize>>) -> Range<usize> {
        range
            .map(|range| self.range_from_utf16(&range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.doc.selection())
    }
}

impl EntityInputHandler for Live {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range);
        *adjusted = Some(self.range_to_utf16(&range));
        Some(self.doc.text[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.doc.selection()),
            reversed: self.doc.reversed(),
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.target(range);
        let text = text.replace("\r\n", "\n");
        self.marked = None;
        let kind = if text.contains('\n') {
            Kind::Other
        } else {
            Kind::Typing
        };
        self.edit(kind, cx, |doc| doc.replace(range, &text));
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.target(range);
        let start = range.start;
        self.edit(Kind::Typing, cx, |doc| doc.replace(range, text));
        self.marked = (!text.is_empty()).then(|| start..start + text.len());
        if let Some(selected) = selected {
            let (from, to) = (
                from_utf16(text, selected.start),
                from_utf16(text, selected.end),
            );
            self.doc.select(start + from, start + to);
        }
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range);
        self.caret_bounds(range.start)
    }

    fn character_index_for_point(
        &mut self,
        at: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(to_utf16(&self.doc.text, self.offset_at(at)))
    }
}

#[cfg(test)]
mod tests {
    use super::{from_utf16, to_utf16};

    #[test]
    fn converts_between_utf8_and_utf16_offsets() {
        let text = "a😀é";
        assert_eq!(to_utf16(text, 5), 3);
        assert_eq!(from_utf16(text, 3), 5);
        assert_eq!(from_utf16(text, 4), 7);
        assert_eq!(from_utf16(text, 9), text.len());
    }
}
