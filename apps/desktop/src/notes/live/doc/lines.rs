use super::{Doc, INDENT, line_start};

impl Doc {
    fn lines(&self) -> Vec<usize> {
        let range = self.selection();
        let mut starts = vec![line_start(&self.text, range.start)];
        let last = if range.end > range.start && self.text[..range.end].ends_with('\n') {
            range.end - 1
        } else {
            range.end
        };
        let mut at = range.start;
        while let Some(found) = self.text[at..last].find('\n') {
            at += found + 1;
            starts.push(at);
        }
        starts
    }

    pub fn indent(&mut self) {
        let starts = self.lines();
        let moved = |offset: usize| {
            offset + INDENT.len() * starts.iter().filter(|start| **start <= offset).count()
        };
        let (anchor, head) = (moved(self.anchor), moved(self.head));
        for start in starts.iter().rev() {
            self.text.insert_str(*start, INDENT);
        }
        self.select(anchor, head);
    }

    pub fn outdent(&mut self) {
        let cuts: Vec<(usize, usize)> = self
            .lines()
            .into_iter()
            .map(|start| {
                let line = &self.text[start..];
                let width = if line.starts_with('\t') {
                    1
                } else {
                    line.bytes()
                        .take(INDENT.len())
                        .take_while(|byte| *byte == b' ')
                        .count()
                };
                (start, width)
            })
            .filter(|(_, width)| *width > 0)
            .collect();
        let moved = |offset: usize| {
            offset
                - cuts
                    .iter()
                    .filter(|(start, _)| *start < offset)
                    .map(|(start, width)| (*width).min(offset - start))
                    .sum::<usize>()
        };
        let (anchor, head) = (moved(self.anchor), moved(self.head));
        for (start, width) in cuts.iter().rev() {
            self.text.replace_range(*start..start + width, "");
        }
        self.select(anchor, head);
    }

    pub fn wrap(&mut self, mark: &str) {
        let range = self.selection();
        let len = mark.len();
        let before =
            range.start >= len && self.text.get(range.start - len..range.start) == Some(mark);
        let after = self.text.get(range.end..range.end + len) == Some(mark);
        if before && after {
            self.text.replace_range(range.end..range.end + len, "");
            self.text.replace_range(range.start - len..range.start, "");
            self.select(range.start - len, range.end - len);
            return;
        }

        self.text.insert_str(range.end, mark);
        self.text.insert_str(range.start, mark);
        self.select(range.start + len, range.end + len);
    }

    pub fn toggle_task(&mut self, at: usize) {
        let mark = match self.text.get(at..at + 3) {
            Some("[ ]") => "x",
            Some("[x]" | "[X]") => " ",
            _ => return,
        };
        let (anchor, head) = (self.anchor, self.head);
        self.text.replace_range(at + 1..at + 2, mark);
        self.select(anchor, head);
    }
}
