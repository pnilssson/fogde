use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wikilink {
    pub range: Range<usize>,
    pub target: String,
    pub label: String,
}

fn code_spans(line: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index] == b'`' {
            index += 1;
        }
        let ticks = index - start;
        match open {
            Some((at, count)) if count == ticks => {
                spans.push(at..index);
                open = None;
            }
            None => open = Some((start, ticks)),
            Some(_) => {}
        }
    }
    spans
}

pub fn parts(inner: &str) -> (String, String) {
    match inner.find('|') {
        None => (inner.trim().to_owned(), inner.to_owned()),
        Some(pipe) => (
            inner[..pipe].trim().to_owned(),
            inner[pipe + 1..].to_owned(),
        ),
    }
}

pub fn find(text: &str) -> Vec<Wikilink> {
    let mut found = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let code = code_spans(line);
        let bytes = line.as_bytes();
        let mut index = 0;
        while index + 1 < bytes.len() {
            let starts = bytes[index] == b'['
                && bytes[index + 1] == b'['
                && (index == 0 || bytes[index - 1] != b'!');
            if !starts || code.iter().any(|span| span.contains(&index)) {
                index += 1;
                continue;
            }
            let inner_start = index + 2;
            let close = line[inner_start..]
                .find([']', '[', '\n', '\r'])
                .map(|at| inner_start + at);
            match close {
                Some(end)
                    if end > inner_start
                        && bytes[end] == b']'
                        && bytes.get(end + 1) == Some(&b']') =>
                {
                    let (target, label) = parts(&line[inner_start..end]);
                    found.push(Wikilink {
                        range: offset + index..offset + end + 2,
                        target,
                        label,
                    });
                    index = end + 2;
                }
                _ => index += 2,
            }
        }
        offset += line.len();
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# Plan\n\nSee [[login]], [[app/plan#Next|the plan]] and ![[diagram.png]].\nCode `[[not a link]]`, an [[unclosed link and a [web link](https://example.com).\n";

    #[test]
    fn parses_wikilinks_but_not_embeds_code_or_unclosed_brackets() {
        let found: Vec<&str> = find(SAMPLE)
            .iter()
            .map(|link| &SAMPLE[link.range.clone()])
            .collect();

        assert_eq!(found, ["[[login]]", "[[app/plan#Next|the plan]]"]);
        assert_eq!(
            parts("app/plan#Next|the plan"),
            ("app/plan#Next".to_owned(), "the plan".to_owned())
        );
    }

    #[test]
    fn skips_empty_links() {
        assert_eq!(find("[[]]"), Vec::new());
    }
}
