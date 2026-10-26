use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::Column;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Term, TermMode};
use alacritty_terminal::vte::ansi::Handler;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const NARROW: char = ' ';
const WIDE: char = '\u{3000}';

pub fn width(grapheme: &str) -> usize {
    if halfwidth_voiced(grapheme) {
        return 2;
    }

    grapheme.width().min(2)
}

pub(crate) fn write<L: EventListener>(term: &mut Term<L>, grapheme: &str) {
    let mut chars = grapheme.chars();
    let Some(first) = chars.next() else {
        return;
    };
    let rest = chars.as_str();
    let width = width(grapheme);

    if width == 0 || (rest.is_empty() && first.width() == Some(width)) {
        for char in grapheme.chars() {
            term.input(char);
        }
        return;
    }

    let grid = term.grid();
    let dropped = width == 2
        && !term.mode().contains(TermMode::LINE_WRAP)
        && grid.cursor.point.column + 1 >= grid.columns();
    term.input(if width == 2 { WIDE } else { NARROW });
    if dropped {
        return;
    }

    let grid = term.grid_mut();
    let line = grid.cursor.point.line;
    let mut column = grid.cursor.point.column;
    if !grid.cursor.input_needs_wrap {
        column = Column(column.0.saturating_sub(1));
    }
    if grid[line][column].flags.contains(Flags::WIDE_CHAR_SPACER) {
        column = Column(column.0.saturating_sub(1));
    }

    let cell = &mut grid[line][column];
    cell.c = first;
    for char in rest.chars() {
        cell.push_zerowidth(char);
    }
}

fn halfwidth_voiced(grapheme: &str) -> bool {
    let mut chars = grapheme.chars();
    let (Some(base), Some(mark), None) = (chars.next(), chars.next(), chars.next()) else {
        return false;
    };

    ('\u{ff66}'..='\u{ff9d}').contains(&base) && matches!(mark, '\u{ff9e}' | '\u{ff9f}')
}
