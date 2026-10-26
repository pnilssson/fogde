use gpui_kit::{App, Pixels, Point};
use terminal::grid::grid::Dimensions;
use terminal::grid::index::Column;
use terminal::grid::term::cell::Flags;

use super::select::grid_point;
use super::view::TerminalView;

const STOPS: &[char] = &[' ', '\0', '"', '\'', '<', '>', '`', '|', '{', '}', '^'];

pub fn find(cells: &[char], column: usize) -> Option<String> {
    if column >= cells.len() || STOPS.contains(&cells[column]) {
        return None;
    }

    let mut start = column;
    while start > 0 && !STOPS.contains(&cells[start - 1]) {
        start -= 1;
    }
    let mut end = column + 1;
    while end < cells.len() && !STOPS.contains(&cells[end]) {
        end += 1;
    }

    let word: String = cells[start..end].iter().collect();
    let at = word.find("http://").or_else(|| word.find("https://"))?;
    let offset = word[..at].chars().count();
    if start + offset > column {
        return None;
    }

    let mut url = &word[at..];
    while let Some(last) = url.chars().last() {
        let strip = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | ']' => true,
            ')' => url.matches('(').count() < url.matches(')').count(),
            _ => false,
        };
        if !strip {
            break;
        }
        url = &url[..url.len() - last.len_utf8()];
    }

    (start + offset + url.chars().count() > column).then(|| url.to_owned())
}

pub fn web(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

pub fn at(view: &TerminalView, position: Point<Pixels>) -> Option<String> {
    let (point, ..) = grid_point(view, position)?;
    let grid = view.screen.term().grid();
    let cell = &grid[point];
    if let Some(link) = cell.hyperlink() {
        return Some(link.uri().to_owned());
    }

    let line = &grid[point.line];
    let cells: Vec<char> = (0..grid.columns())
        .map(|column| {
            let cell = &line[Column(column)];
            if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                '\0'
            } else {
                cell.c
            }
        })
        .collect();
    find(&cells, point.column.0)
}

pub fn open(url: &str, cx: &mut App) {
    if !web(url) {
        return;
    }
    cx.open_url(url);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn finds_the_url_under_the_pointer() {
        let cells = line("see https://example.com/login. now");
        assert_eq!(
            find(&cells, 10).as_deref(),
            Some("https://example.com/login")
        );
        assert_eq!(find(&cells, 2), None);
        assert_eq!(find(&cells, 31), None);
    }

    #[test]
    fn keeps_balanced_parentheses() {
        let cells = line("(https://en.wikipedia.org/wiki/Rust_(language))");
        assert_eq!(
            find(&cells, 5).as_deref(),
            Some("https://en.wikipedia.org/wiki/Rust_(language)")
        );
    }

    #[test]
    fn opens_only_web_links() {
        assert!(web("HTTPS://example.com"));
        assert!(!web("file:///etc/passwd"));
        assert!(!web("javascript:alert(1)"));
    }
}
