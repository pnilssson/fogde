use gpui_kit::A11ySubtreeBuilder;
use gpui_kit::accesskit::{Live, Node, Role, TextPosition, TextSelection};
use terminal::Screen;

pub struct Rows {
    pub label: String,
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
}

pub fn snapshot(screen: &Screen, label: String) -> Rows {
    let term = screen.term();
    let grid = term.grid();
    let rows = terminal::grid::grid::Dimensions::screen_lines(grid);
    let point = grid.cursor.point;
    Rows {
        label,
        lines: (0..rows).map(|row| screen.text(row)).collect(),
        cursor: (
            usize::try_from(point.line.0).unwrap_or_default(),
            point.column.0,
        ),
    }
}

pub fn describe(rows: &Rows, node: &mut Node) {
    node.set_label(rows.label.clone());
    node.set_live(Live::Polite);
}

pub fn children(rows: &Rows, builder: &mut A11ySubtreeBuilder) {
    let mut cursor = None;
    for (index, line) in rows.lines.iter().enumerate() {
        let mut run = Node::new(Role::TextRun);
        run.set_value(line.clone());
        let lengths: Vec<u8> = line
            .chars()
            .map(|char| u8::try_from(char.len_utf8()).unwrap_or(1))
            .collect();
        run.set_character_lengths(lengths);
        let id = builder.synthetic_node_id(index);
        builder.push_child(id, run);
        if index == rows.cursor.0 {
            let column = rows.cursor.1.min(line.chars().count());
            cursor = Some(TextPosition {
                node: id,
                character_index: column,
            });
        }
    }

    if let Some(caret) = cursor {
        builder.parent_node().set_text_selection(TextSelection {
            anchor: caret,
            focus: caret,
        });
    }
}
