use std::ops::Range;

use markdown::mdast::Node;
use markdown::{ParseOptions, to_mdast};

use super::doc::{line_end, line_start};
use crate::notes::wikilinks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Body,
    Heading(u8),
    Code,
    Quote,
    Table,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    Strong,
    Emphasis,
    Strike,
    Code,
    Muted,
    Link(String),
    Wikilink(String),
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub kinds: Vec<(Range<usize>, Kind)>,
    pub hidden: Vec<Range<usize>>,
    pub marks: Vec<(Range<usize>, Mark)>,
    pub bullets: Vec<usize>,
    pub tasks: Vec<(usize, bool)>,
    pub rules: Vec<Range<usize>>,
}

fn span(node: &Node) -> Option<Range<usize>> {
    node.position()
        .map(|position| position.start.offset..position.end.offset)
}

fn inner(node: &Node) -> Option<Range<usize>> {
    let children = node.children()?;
    let first = span(children.first()?)?;
    let last = span(children.last()?)?;
    Some(first.start..last.end)
}

impl Parsed {
    fn hide(&mut self, range: Range<usize>) {
        if range.start < range.end {
            self.marks.push((range.clone(), Mark::Muted));
            self.hidden.push(range);
        }
    }

    fn around(&mut self, node: &Node, range: &Range<usize>) {
        if let Some(inner) = inner(node) {
            self.hide(range.start..inner.start);
            self.hide(inner.end..range.end);
        }
    }

    fn quotes(&mut self, text: &str, range: &Range<usize>) {
        let mut start = line_start(text, range.start);
        while start <= range.end && start < text.len() {
            let end = line_end(text, start);
            let line = &text[start..end];
            let mut at = start + line.len() - line.trim_start_matches(' ').len();
            while text[at..end].starts_with('>') {
                let mark = if text[at + 1..end].starts_with(' ') {
                    2
                } else {
                    1
                };
                self.hide(at..at + mark);
                at += mark;
            }
            start = end + 1;
        }
    }

    fn heading(&mut self, text: &str, node: &Node, range: &Range<usize>, depth: u8) {
        self.kinds.push((range.clone(), Kind::Heading(depth)));
        let inner = inner(node);
        if !text[range.clone()].trim_start().starts_with('#') {
            self.hide(line_start(text, range.end)..range.end);
            return;
        }

        self.hide(range.start..inner.as_ref().map_or(range.end, |inner| inner.start));
        if let Some(inner) = inner.filter(|inner| inner.end < range.end) {
            self.marks.push((inner.end..range.end, Mark::Muted));
        }
    }

    fn item(&mut self, text: &str, range: &Range<usize>, checked: Option<bool>) {
        if matches!(text[range.start..].chars().next(), Some('-' | '*' | '+')) {
            self.bullets.push(range.start);
        }
        let Some(checked) = checked else {
            return;
        };
        let end = line_end(text, range.start);
        if let Some(open) = text[range.start..end].find('[') {
            self.tasks.push((range.start + open, checked));
        }
    }

    fn link(&mut self, text: &str, node: &Node, range: &Range<usize>, url: &str) {
        if !text[range.start..].starts_with('[') {
            self.marks.push((range.clone(), Mark::Link(url.to_owned())));
            return;
        }
        if let Some(inner) = inner(node) {
            self.marks.push((inner.clone(), Mark::Link(url.to_owned())));
            self.around(node, range);
        }
    }

    fn walk(&mut self, text: &str, node: &Node) {
        if let Some(range) = span(node) {
            match node {
                Node::Heading(heading) => self.heading(text, node, &range, heading.depth),
                Node::Blockquote(_) => {
                    self.kinds.push((range.clone(), Kind::Quote));
                    self.quotes(text, &range);
                }
                Node::Code(_) => {
                    self.kinds.push((range, Kind::Code));
                    return;
                }
                Node::Table(_) => self.kinds.push((range.clone(), Kind::Table)),
                Node::ThematicBreak(_) => {
                    self.marks.push((range.clone(), Mark::Muted));
                    self.rules.push(range);
                }
                Node::ListItem(item) => self.item(text, &range, item.checked),
                Node::Emphasis(_) => {
                    self.marks.push((range.clone(), Mark::Emphasis));
                    self.around(node, &range);
                }
                Node::Strong(_) => {
                    self.marks.push((range.clone(), Mark::Strong));
                    self.around(node, &range);
                }
                Node::Delete(_) => {
                    self.marks.push((range.clone(), Mark::Strike));
                    self.around(node, &range);
                }
                Node::InlineCode(_) => {
                    let ticks = text[range.clone()]
                        .bytes()
                        .take_while(|byte| *byte == b'`')
                        .count();
                    self.marks.push((range.clone(), Mark::Code));
                    self.hide(range.start..range.start + ticks);
                    self.hide(range.end - ticks..range.end);
                    return;
                }
                Node::Link(link) => self.link(text, node, &range, &link.url),
                _ => {}
            }
        }

        for child in node.children().into_iter().flatten() {
            self.walk(text, child);
        }
    }

    fn coded(&self, offset: usize) -> bool {
        self.kinds
            .iter()
            .any(|(range, kind)| *kind == Kind::Code && range.contains(&offset))
            || self
                .marks
                .iter()
                .any(|(range, mark)| *mark == Mark::Code && range.contains(&offset))
    }
}

pub fn parse(text: &str) -> Parsed {
    let mut parsed = Parsed::default();
    if let Ok(root) = to_mdast(text, &ParseOptions::gfm()) {
        parsed.walk(text, &root);
    }

    for link in wikilinks::find(text) {
        if parsed.coded(link.range.start) {
            continue;
        }
        let close = link.range.end - 2;
        let label = close - link.label.len();
        parsed.hide(link.range.start..label);
        parsed
            .marks
            .push((label..close, Mark::Wikilink(link.target)));
        parsed.hide(close..link.range.end);
    }
    parsed
}
