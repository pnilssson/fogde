use super::{Cursor, Layout, cell_bounds};
use crate::terminals::view::Cell;
use gpui_kit::{Bounds, Hsla, Pixels, Point};
use terminal::grid::grid::Grid;
use terminal::grid::selection::SelectionRange;
use terminal::grid::term::RenderableCursor;
use terminal::grid::term::cell::{Cell as GridCell, Flags};
use terminal::grid::vte::ansi::CursorShape;

pub(super) fn selection(
    range: &SelectionRange,
    rows: usize,
    columns: usize,
    origin: Point<Pixels>,
    cell: Cell,
) -> Vec<Bounds<Pixels>> {
    let mut bounds = Vec::new();
    for row in 0..rows {
        let line = i32::try_from(row).unwrap_or(i32::MAX);
        if line < range.start.line.0 || line > range.end.line.0 {
            continue;
        }
        let start = if line == range.start.line.0 {
            range.start.column.0
        } else {
            0
        };
        let end = if line == range.end.line.0 {
            range.end.column.0 + 1
        } else {
            columns
        };
        if end > start {
            bounds.push(cell_bounds(origin, cell, start, row, end - start));
        }
    }
    bounds
}

pub(super) fn flush(
    layout: &mut Layout,
    background: Option<(usize, usize, Hsla)>,
    origin: Point<Pixels>,
    cell: Cell,
    row: usize,
) {
    if let Some((start, end, color)) = background {
        layout
            .backgrounds
            .push((cell_bounds(origin, cell, start, row, end - start), color));
    }
}

pub(super) fn cursor(
    grid: &Grid<GridCell>,
    cursor: RenderableCursor,
    origin: Point<Pixels>,
    cell: Cell,
) -> Option<Cursor> {
    if cursor.shape == CursorShape::Hidden {
        return None;
    }

    let row = usize::try_from(cursor.point.line.0).ok()?;
    let wide = grid[cursor.point].flags.contains(Flags::WIDE_CHAR);
    Some(Cursor {
        bounds: cell_bounds(
            origin,
            cell,
            cursor.point.column.0,
            row,
            if wide { 2 } else { 1 },
        ),
        shape: cursor.shape,
    })
}
