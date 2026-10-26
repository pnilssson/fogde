mod lines;

#[cfg(test)]
mod tests;

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

pub const INDENT: &str = "  ";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Doc {
    pub text: String,
    pub anchor: usize,
    pub head: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Word,
    Space,
    Other,
}

fn class(char: char) -> Class {
    if char.is_alphanumeric() || char == '_' {
        Class::Word
    } else if char.is_whitespace() {
        Class::Space
    } else {
        Class::Other
    }
}

pub fn previous(text: &str, offset: usize) -> usize {
    text[..offset]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(at, _)| at)
}

pub fn next(text: &str, offset: usize) -> usize {
    text[offset..]
        .graphemes(true)
        .next()
        .map_or(offset, |grapheme| offset + grapheme.len())
}

pub fn word_left(text: &str, offset: usize) -> usize {
    let mut chars = text[..offset].char_indices().rev().peekable();
    let mut at = offset;
    while let Some((index, _)) = chars.next_if(|(_, char)| *char != '\n' && char.is_whitespace()) {
        at = index;
    }
    let Some(&(index, first)) = chars.peek() else {
        return at;
    };
    if first == '\n' {
        return if at == offset { index } else { at };
    }

    let kind = class(first);
    while let Some((index, _)) = chars.next_if(|(_, char)| *char != '\n' && class(*char) == kind) {
        at = index;
    }
    at
}

pub fn word_right(text: &str, offset: usize) -> usize {
    let mut chars = text[offset..].char_indices().peekable();
    let mut at = offset;
    while let Some((index, char)) = chars.next_if(|(_, char)| *char != '\n' && char.is_whitespace())
    {
        at = offset + index + char.len_utf8();
    }
    let Some(&(index, first)) = chars.peek() else {
        return at;
    };
    if first == '\n' {
        return if at == offset { offset + index + 1 } else { at };
    }

    let kind = class(first);
    while let Some((index, char)) = chars.next_if(|(_, char)| *char != '\n' && class(*char) == kind)
    {
        at = offset + index + char.len_utf8();
    }
    at
}

pub fn line_start(text: &str, offset: usize) -> usize {
    text[..offset].rfind('\n').map_or(0, |at| at + 1)
}

pub fn line_end(text: &str, offset: usize) -> usize {
    text[offset..]
        .find('\n')
        .map_or(text.len(), |at| offset + at)
}

struct Markup {
    len: usize,
    next: String,
}

fn markup(line: &str) -> Option<Markup> {
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let mut at = indent;
    let mut quotes = String::new();
    while line[at..].starts_with('>') {
        let mark = if line[at + 1..].starts_with(' ') {
            2
        } else {
            1
        };
        quotes.push_str("> ");
        at += mark;
    }

    let rest = &line[at..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let (marker, next) = if rest.starts_with("- ")
        || rest.starts_with("* ")
        || rest.starts_with("+ ")
    {
        (2, rest[..2].to_owned())
    } else if digits > 0 && (rest[digits..].starts_with(". ") || rest[digits..].starts_with(") ")) {
        let number: u64 = rest[..digits].parse().ok()?;
        (
            digits + 2,
            format!("{}{}", number + 1, &rest[digits..digits + 2]),
        )
    } else {
        (0, String::new())
    };
    let task = &rest[marker..];
    let checkbox = marker > 0
        && (task.starts_with("[ ] ") || task.starts_with("[x] ") || task.starts_with("[X] "));
    let len = at + marker + if checkbox { 4 } else { 0 };
    if len == indent {
        return None;
    }

    let mut next = format!("{}{quotes}{next}", &line[..indent]);
    if checkbox {
        next.push_str("[ ] ");
    }
    Some(Markup { len, next })
}

impl Doc {
    pub fn new(text: String) -> Self {
        Self {
            text,
            anchor: 0,
            head: 0,
        }
    }

    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    pub fn reversed(&self) -> bool {
        self.head < self.anchor
    }

    pub fn collapsed(&self) -> bool {
        self.anchor == self.head
    }

    pub fn selected(&self) -> &str {
        &self.text[self.selection()]
    }

    pub fn clamp(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    pub fn select(&mut self, anchor: usize, head: usize) {
        self.anchor = self.clamp(anchor);
        self.head = self.clamp(head);
    }

    pub fn place(&mut self, head: usize, extend: bool) {
        let head = self.clamp(head);
        self.head = head;
        if !extend {
            self.anchor = head;
        }
    }

    pub fn replace(&mut self, range: Range<usize>, insert: &str) {
        let range = self.clamp(range.start)..self.clamp(range.end);
        self.text.replace_range(range.clone(), insert);
        let caret = range.start + insert.len();
        self.anchor = caret;
        self.head = caret;
    }

    pub fn insert(&mut self, insert: &str) {
        self.replace(self.selection(), insert);
    }

    pub fn delete_backward(&mut self) {
        if !self.collapsed() {
            self.insert("");
            return;
        }

        let start = line_start(&self.text, self.head);
        if let Some(markup) = markup(&self.text[start..self.head])
            && start + markup.len == self.head
        {
            self.replace(start..self.head, "");
            return;
        }
        self.replace(previous(&self.text, self.head)..self.head, "");
    }

    pub fn delete_forward(&mut self) {
        if self.collapsed() {
            self.replace(self.head..next(&self.text, self.head), "");
        } else {
            self.insert("");
        }
    }

    pub fn delete_to(&mut self, target: usize) {
        if self.collapsed() {
            let range = target.min(self.head)..target.max(self.head);
            self.replace(range, "");
        } else {
            self.insert("");
        }
    }

    pub fn newline(&mut self) {
        self.insert("");
        let start = line_start(&self.text, self.head);
        let end = line_end(&self.text, self.head);
        let Some(markup) =
            markup(&self.text[start..end]).filter(|markup| start + markup.len <= self.head)
        else {
            self.insert("\n");
            return;
        };

        if self.text[start + markup.len..end].trim().is_empty() {
            self.replace(start..end, "");
        } else {
            self.insert(&format!("\n{}", markup.next));
        }
    }
}
