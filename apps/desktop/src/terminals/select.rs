use gpui_kit::{ClipboardItem, Context, Pixels, Point, ScrollEvent};
use protocol::terminals::ScrollTerminalParams;
use terminal::grid::grid::Dimensions;
use terminal::grid::index::{Column, Line, Point as GridPoint, Side};
use terminal::grid::selection::{Selection, SelectionType};

use super::outgoing::Outgoing;
use super::scroll;
use super::view::TerminalView;

pub fn grid_point(
    view: &TerminalView,
    position: Point<Pixels>,
) -> Option<(GridPoint, Side, u16, u16)> {
    let (bounds, cell) = (view.bounds?, view.cell?);
    let term = view.screen.term();
    let columns = u16::try_from(term.columns()).unwrap_or(u16::MAX);
    let rows = u16::try_from(term.screen_lines()).unwrap_or(u16::MAX);
    let x = position.x - bounds.origin.x;
    let y = position.y - bounds.origin.y;
    let column = scroll::cell(x.into(), (cell.width * f32::from(columns)).into(), columns);
    let row = scroll::cell(y.into(), (cell.height * f32::from(rows)).into(), rows);
    let within = x - cell.width * f32::from(column);
    let side = if within > cell.width / 2. {
        Side::Right
    } else {
        Side::Left
    };

    Some((
        GridPoint::new(Line(i32::from(row)), Column(usize::from(column))),
        side,
        column,
        row,
    ))
}

pub fn start(view: &mut TerminalView, position: Point<Pixels>, clicks: usize) {
    let Some((point, side, ..)) = grid_point(view, position) else {
        return;
    };
    let kind = match clicks {
        2 => SelectionType::Semantic,
        3.. => SelectionType::Lines,
        _ => SelectionType::Simple,
    };

    view.screen.term_mut().selection = Some(Selection::new(kind, point, side));
    view.selecting = true;
}

pub fn extend(view: &mut TerminalView, position: Point<Pixels>) {
    let Some((point, side, ..)) = grid_point(view, position) else {
        return;
    };
    if let Some(selection) = &mut view.screen.term_mut().selection {
        selection.update(point, side);
    }
}

pub fn selected(view: &TerminalView) -> Option<String> {
    view.screen
        .term()
        .selection_to_string()
        .filter(|text| !text.is_empty())
}

pub fn clear(view: &mut TerminalView) {
    view.screen.term_mut().selection = None;
}

pub fn copy(view: &mut TerminalView, cx: &mut Context<TerminalView>) {
    let Some(text) = selected(view) else {
        clear(view);
        cx.notify();
        return;
    };

    cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
    #[cfg(target_os = "linux")]
    cx.write_to_primary(ClipboardItem::new_string(text.clone()));
    let _ = text;
    view.flash_copied(cx);
}

pub fn paste(view: &mut TerminalView, cx: &mut Context<TerminalView>) {
    let Some(item) = cx.read_from_clipboard() else {
        return;
    };
    if let Some(text) = item.text() {
        view.send_text(&terminal::bracket(&text));
        return;
    }

    super::images::attach(view, item, cx);
}

pub fn wheel(view: &mut TerminalView, event: &ScrollEvent) {
    let Some(cell) = view.cell else {
        return;
    };
    let delta = event.delta.pixel_delta(cell.height);
    let Some(step) = view.wheel.add(-f32::from(delta.y), cell.height.into()) else {
        return;
    };
    let Some((_, _, column, row)) = grid_point(view, event.position) else {
        return;
    };

    view.send(Outgoing::Scroll(ScrollTerminalParams {
        id: String::new(),
        direction: step.direction,
        lines: step.lines,
        column,
        row,
    }));
}
