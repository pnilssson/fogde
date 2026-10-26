mod parts;

use parts::{cursor, flush, selection};

use std::sync::Arc;

use gpui_kit::{
    Bounds, Font, FontFeatures, FontStyle, FontWeight, Hsla, Pixels, Point, ShapedLine,
    SharedString, StrikethroughStyle, TextRun, UnderlineStyle, Window, point, px, size,
};
use terminal::grid::grid::Dimensions;
use terminal::grid::index::{Column, Line};
use terminal::grid::term::cell::{Cell as GridCell, Flags};
use terminal::grid::vte::ansi::CursorShape;

use super::boxes;
use super::view::{Cell, TerminalView};
use crate::num;
use crate::theme::MONO;

pub const FONT_SIZE: Pixels = px(13.);
pub const LINE_HEIGHT: Pixels = px(20.);

pub fn font(bold: bool, italic: bool) -> Font {
    Font {
        family: MONO.into(),
        features: FontFeatures(Arc::new(vec![
            ("calt".into(), 0),
            ("liga".into(), 0),
            ("dlig".into(), 0),
        ])),
        fallbacks: None,
        weight: if bold {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        },
        style: if italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
    }
}

pub fn measure(window: &Window) -> Cell {
    let system = window.text_system();
    let id = system.resolve_font(&font(false, false));
    let width = system
        .advance(id, FONT_SIZE, 'm')
        .map_or(px(8.), |advance| advance.width);
    Cell {
        width,
        height: LINE_HEIGHT,
    }
}

pub struct Glyphs {
    pub origin: Point<Pixels>,
    pub line: ShapedLine,
}

pub struct Drawn {
    pub char: char,
    pub bounds: Bounds<Pixels>,
    pub color: Hsla,
}

pub struct Cursor {
    pub bounds: Bounds<Pixels>,
    pub shape: CursorShape,
}

#[derive(Default)]
pub struct Layout {
    pub backgrounds: Vec<(Bounds<Pixels>, Hsla)>,
    pub glyphs: Vec<Glyphs>,
    pub drawn: Vec<Drawn>,
    pub selection: Vec<Bounds<Pixels>>,
    pub cursor: Option<Cursor>,
}

struct Run {
    text: String,
    runs: Vec<TextRun>,
    column: usize,
}

fn cell_bounds(
    origin: Point<Pixels>,
    cell: Cell,
    column: usize,
    row: usize,
    width: usize,
) -> Bounds<Pixels> {
    Bounds::new(
        point(
            origin.x + cell.width * num::real_usize(column),
            origin.y + cell.height * num::real_usize(row),
        ),
        size(cell.width * num::real_usize(width), cell.height),
    )
}

fn text_run(len: usize, cell: &GridCell, color: Hsla) -> TextRun {
    let underline = cell
        .flags
        .intersects(Flags::ALL_UNDERLINES)
        .then_some(UnderlineStyle {
            thickness: px(1.),
            color: Some(color),
            wavy: cell.flags.contains(Flags::UNDERCURL),
        });
    TextRun {
        len,
        font: font(
            cell.flags.contains(Flags::BOLD),
            cell.flags.contains(Flags::ITALIC),
        ),
        color,
        background_color: None,
        underline,
        strikethrough: cell
            .flags
            .contains(Flags::STRIKEOUT)
            .then_some(StrikethroughStyle {
                thickness: px(1.),
                color: Some(color),
            }),
    }
}

pub fn layout(
    view: &TerminalView,
    origin: Point<Pixels>,
    cell: Cell,
    window: &mut Window,
) -> Layout {
    let term = view.screen.term();
    let content = term.renderable_content();
    let colors = content.colors;
    let grid = term.grid();
    let rows = grid.screen_lines();
    let columns = grid.columns();
    let mut layout = Layout::default();

    let shape = |run: Option<Run>, row: usize, glyphs: &mut Vec<Glyphs>, force: bool| {
        let Some(run) = run else {
            return;
        };
        if run.text.trim().is_empty() {
            return;
        }
        let line = window.text_system().shape_line(
            SharedString::from(run.text),
            FONT_SIZE,
            &run.runs,
            force.then_some(cell.width),
        );
        glyphs.push(Glyphs {
            origin: cell_bounds(origin, cell, run.column, row, 1).origin,
            line,
        });
    };

    for row in 0..rows {
        let line = &grid[Line(i32::try_from(row).unwrap_or(i32::MAX))];
        let mut background: Option<(usize, usize, Hsla)> = None;
        let mut run: Option<Run> = None;

        for column in 0..columns {
            let grid_cell = &line[Column(column)];
            if grid_cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            let width = usize::from(grid_cell.flags.contains(Flags::WIDE_CHAR)) + 1;
            let (foreground, fill) =
                view.palette
                    .cell(grid_cell.fg, grid_cell.bg, grid_cell.flags, colors);

            match (&mut background, fill) {
                (Some((_, end, color)), Some(fill)) if *end == column && *color == fill => {
                    *end = column + width;
                }
                (_, fill) => {
                    flush(&mut layout, background.take(), origin, cell, row);
                    background = fill.map(|fill| (column, column + width, fill));
                }
            }

            let char = grid_cell.c;
            if char == ' ' || char == '\0' {
                if let Some(current) = &mut run {
                    current.text.push(' ');
                    current.runs.push(text_run(1, grid_cell, foreground));
                }
                continue;
            }

            if boxes::drawn(char) {
                shape(run.take(), row, &mut layout.glyphs, true);
                layout.drawn.push(Drawn {
                    char,
                    bounds: cell_bounds(origin, cell, column, row, 1),
                    color: foreground,
                });
                continue;
            }

            let mut text = String::from(char);
            text.extend(grid_cell.zerowidth().into_iter().flatten());
            if width == 2 {
                shape(run.take(), row, &mut layout.glyphs, true);
                shape(
                    Some(Run {
                        runs: vec![text_run(text.len(), grid_cell, foreground)],
                        text,
                        column,
                    }),
                    row,
                    &mut layout.glyphs,
                    false,
                );
                continue;
            }

            let current = run.get_or_insert_with(|| Run {
                text: String::new(),
                runs: Vec::new(),
                column,
            });
            current
                .runs
                .push(text_run(text.len(), grid_cell, foreground));
            current.text.push_str(&text);
        }

        flush(&mut layout, background, origin, cell, row);
        shape(run.take(), row, &mut layout.glyphs, true);
    }

    if let Some(range) = content.selection {
        layout.selection = selection(&range, rows, columns, origin, cell);
    }

    layout.cursor = cursor(grid, content.cursor, origin, cell);

    layout
}

pub fn paint_drawn(drawn: &[Drawn], window: &mut Window) {
    for item in drawn {
        boxes::paint(item.char, item.bounds, item.color, window);
    }
}
