#[cfg(test)]
mod tests;

use std::ops::Range;

use super::parse::{Kind, Mark, Parsed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Widget {
    None,
    Rule,
    Bullet,
    Task { checked: bool, at: usize },
}

#[derive(Clone, Debug)]
pub struct Line {
    pub range: Range<usize>,
    pub kind: Kind,
    pub widget: Widget,
    pub indent: usize,
    pub text: String,
    pub map: Vec<(usize, usize)>,
    pub marks: Vec<(Range<usize>, Mark)>,
}

impl Line {
    pub fn source(&self, shown: usize) -> usize {
        self.map
            .iter()
            .rev()
            .find(|(start, _)| *start <= shown)
            .map_or(self.range.start, |(start, source)| source + (shown - start))
            .min(self.range.end)
    }

    pub fn shown(&self, source: usize) -> Option<usize> {
        let pieces = self.map.iter().zip(
            self.map
                .iter()
                .skip(1)
                .map(|(start, _)| *start)
                .chain(std::iter::once(self.text.len())),
        );
        for ((start, from), end) in pieces {
            if (*from..from + (end - start)).contains(&source) || source == from + (end - start) {
                return Some(start + (source - from));
            }
        }
        None
    }
}

pub fn ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (at, _) in text.match_indices('\n') {
        ranges.push(start..at);
        start = at + 1;
    }
    ranges.push(start..text.len());
    ranges
}

fn touches(kind: &Range<usize>, line: &Range<usize>) -> bool {
    kind.start <= line.end && kind.end >= line.start && kind.start < kind.end
}

fn kind(parsed: &Parsed, line: &Range<usize>) -> Kind {
    let found: Vec<Kind> = parsed
        .kinds
        .iter()
        .filter(|(range, _)| touches(range, line))
        .map(|(_, kind)| *kind)
        .collect();
    let rank = |kind: &Kind| match kind {
        Kind::Code => 0,
        Kind::Heading(_) => 1,
        Kind::Table => 2,
        Kind::Quote => 3,
        Kind::Body => 4,
    };
    found.into_iter().min_by_key(rank).unwrap_or(Kind::Body)
}

fn clip(range: &Range<usize>, within: &Range<usize>) -> Option<Range<usize>> {
    let clipped = range.start.max(within.start)..range.end.min(within.end);
    (clipped.start < clipped.end).then_some(clipped)
}

fn visible(line: &Range<usize>, hidden: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut cuts: Vec<Range<usize>> = hidden
        .iter()
        .filter_map(|range| clip(range, line))
        .collect();
    cuts.sort_by_key(|range| range.start);
    let mut pieces = Vec::new();
    let mut at = line.start;
    for cut in cuts {
        if cut.start > at {
            pieces.push(at..cut.start);
        }
        at = at.max(cut.end);
    }
    if at < line.end {
        pieces.push(at..line.end);
    }
    pieces
}

fn columns(text: &str) -> usize {
    text.chars()
        .map(|char| match char {
            '\t' => 4,
            ' ' => 1,
            _ => 0,
        })
        .sum()
}

fn widget(text: &str, parsed: &Parsed, line: &Range<usize>) -> Option<(Widget, usize, usize)> {
    let bullet = parsed
        .bullets
        .iter()
        .copied()
        .find(|at| line.contains(at))?;
    let after = |at: usize| at + usize::from(text[at..line.end].starts_with(' '));
    let mut end = after(bullet + 1);
    let mut widget = Widget::Bullet;
    if let Some((at, checked)) = parsed
        .tasks
        .iter()
        .find(|(at, _)| line.contains(at) && *at > bullet)
    {
        end = after(at + 3).min(line.end);
        widget = Widget::Task {
            checked: *checked,
            at: *at,
        };
    }

    let lead: String = visible(&(line.start..bullet), &parsed.hidden)
        .into_iter()
        .map(|piece| &text[piece])
        .collect();
    if !lead.trim().is_empty() {
        return None;
    }
    Some((widget, columns(&lead), end))
}

type Shown = (String, Vec<(usize, usize)>, Vec<(Range<usize>, Mark)>);

fn shown(text: &str, parsed: &Parsed, line: &Range<usize>, hidden: &[Range<usize>]) -> Shown {
    let pieces = visible(line, hidden);
    let mut shown = String::new();
    let mut map = Vec::new();
    for piece in &pieces {
        map.push((shown.len(), piece.start));
        shown.push_str(&text[piece.clone()]);
    }
    if map.is_empty() {
        map.push((
            0,
            hidden
                .iter()
                .filter_map(|range| clip(range, line))
                .map(|range| range.end)
                .max()
                .unwrap_or(line.start),
        ));
    }

    let mut marks = Vec::new();
    for (range, mark) in &parsed.marks {
        for ((start, source), piece) in map.iter().zip(&pieces) {
            if let Some(part) = clip(range, piece) {
                let from = start + (part.start - source);
                marks.push((from..from + part.len(), mark.clone()));
            }
        }
    }
    (shown, map, marks)
}

pub fn lines(text: &str, parsed: &Parsed, active: &Range<usize>) -> Vec<Line> {
    ranges(text)
        .into_iter()
        .enumerate()
        .map(|(index, range)| {
            let kind = kind(parsed, &range);
            let editing = active.contains(&index);
            let rule = parsed.rules.iter().any(|rule| touches(rule, &range));
            let mut line = Line {
                range: range.clone(),
                kind,
                widget: Widget::None,
                indent: 0,
                text: String::new(),
                map: vec![(0, range.start)],
                marks: Vec::new(),
            };
            if !editing && rule {
                line.widget = Widget::Rule;
                return line;
            }

            let mut hidden = if editing {
                Vec::new()
            } else {
                parsed.hidden.clone()
            };
            if !editing && let Some((widget, indent, end)) = widget(text, parsed, &range) {
                hidden.push(range.start..end);
                line.widget = widget;
                line.indent = indent;
            }
            let (shown, map, marks) = shown(text, parsed, &range, &hidden);
            line.text = shown;
            line.map = map;
            line.marks = marks;
            line
        })
        .collect()
}
