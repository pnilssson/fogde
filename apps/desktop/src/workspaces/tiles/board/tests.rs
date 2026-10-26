mod notes;

use std::cell::Cell;

use super::Board;
use crate::workspaces::tiles::geometry::{Rect, Side, Zone};
use crate::workspaces::tiles::tile::{Dragged, START, Selection, Tile, View};

const AREA: Rect = Rect {
    x: 0.,
    y: 0.,
    width: 900.,
    height: 600.,
};

fn select(machine: &str, id: &str) -> Selection {
    Selection::new(machine, id)
}

fn board() -> Board {
    let mut board = Board::default();
    board.select(View::workspace("m1", "w1"));
    board
}

fn ids(board: &Board) -> Vec<String> {
    board.layout().tiles().into_iter().map(Tile::id).collect()
}

fn rect(board: &Board, id: &str) -> Rect {
    board
        .layout()
        .rects(AREA)
        .into_iter()
        .find(|(known, _)| known == id)
        .unwrap()
        .1
}

fn note(board: &Board, id: &str) -> Option<String> {
    match board.layout().tile(id) {
        Some(Tile::Notes { note, .. }) => note.clone(),
        _ => None,
    }
}

fn terminals(names: &[&str]) -> Vec<Selection> {
    names.iter().map(|name| select("m1", name)).collect()
}

fn key() -> String {
    let next = Cell::new(0);
    next.set(next.get() + 1);
    format!("notes:{}", next.get())
}

#[test]
fn dropping_a_tile_on_another_tiles_centre_swaps_them() {
    let mut board = board();
    board.sync(&terminals(&["a", "b", "c"]), |_| false);

    board.drop(
        &Dragged::Terminal(select("m1", "a")),
        Some("m1:c"),
        Zone::Center,
        key,
    );

    assert_eq!(ids(&board), ["m1:c", "m1:b", "m1:a"]);
}

#[test]
fn every_terminal_in_the_workspace_gets_its_own_tile() {
    let mut board = board();

    board.sync(&terminals(&["a", "b", "c"]), |_| false);
    assert_eq!(ids(&board), ["m1:a", "m1:b", "m1:c"]);

    board.sync(&terminals(&["a", "c"]), |_| false);
    assert_eq!(ids(&board), ["m1:a", "m1:c"]);
}

#[test]
fn a_terminal_being_placed_keeps_its_tile() {
    let mut board = board();
    board.sync(&terminals(&["a", "b"]), |_| false);

    board.sync(&terminals(&["a"]), |selection| selection.id == "b");

    assert_eq!(ids(&board), ["m1:a", "m1:b"]);
}

#[test]
fn dropping_a_tile_on_the_outer_edge_spans_the_whole_side() {
    let mut board = board();
    board.sync(&terminals(&["a", "b", "c"]), |_| false);

    board.drop(
        &Dragged::Terminal(select("m1", "a")),
        None,
        Zone::Edge(Side::Bottom),
        key,
    );

    assert!((rect(&board, "m1:a").width - 900.).abs() < 0.01);
    assert!(rect(&board, "m1:b").width < 900.);
}

#[test]
fn a_dragged_tile_drops_only_where_it_would_move() {
    let mut board = board();
    board.sync(&terminals(&["a", "b"]), |_| false);
    let a = Dragged::Terminal(select("m1", "a"));
    let c = Dragged::Terminal(select("m1", "c"));

    assert!(!board.can_drop(&a, Some("m1:a"), Zone::Edge(Side::Right)));
    assert!(board.can_drop(&a, Some("m1:b"), Zone::Center));
    assert!(board.can_drop(&a, None, Zone::Edge(Side::Bottom)));

    assert!(!board.can_drop(&c, Some("m1:b"), Zone::Center));
    assert!(board.can_drop(&c, Some("m1:b"), Zone::Edge(Side::Left)));

    let other = Dragged::Terminal(select("m2", "d"));
    assert!(!board.can_drop(&other, Some("m1:b"), Zone::Edge(Side::Left)));
}

#[test]
fn opening_a_workspace_focuses_only_its_first_terminal() {
    let mut board = board();

    board.sync(&terminals(&["a", "b", "c"]), |_| false);

    assert_eq!(board.active_terminal(), Some(select("m1", "a")));
}

#[test]
fn syncing_terminals_leaves_other_tiles_open() {
    let mut board = board();
    board.sync(&terminals(&["a", "b"]), |_| false);
    board.open_start("/repos/app");

    board.sync(&terminals(&["a"]), |_| false);

    assert_eq!(ids(&board), ["m1:a", START]);
}

#[test]
fn another_tile_moves_by_its_id_and_drops_only_where_it_exists() {
    let mut board = board();
    board.sync(&terminals(&["a", "b"]), |_| false);
    board.open_start("/repos/app");
    let start = Dragged::Tile {
        id: START.into(),
        view: None,
    };
    let gone = Dragged::Tile {
        id: "notes:2".into(),
        view: None,
    };

    assert!(board.can_drop(&start, Some("m1:a"), Zone::Center));
    assert!(!board.can_drop(&gone, Some("m1:a"), Zone::Edge(Side::Left)));

    board.drop(&start, Some("m1:a"), Zone::Center, key);

    assert_eq!(ids(&board), [START, "m1:a", "m1:b"]);
}

#[test]
fn focusing_another_tile_clears_the_active_terminal() {
    let mut board = board();
    board.sync(&terminals(&["a"]), |_| false);
    board.open_start("/repos/app");

    board.activate(START);

    assert_eq!(board.active(), Some(START));
    assert_eq!(board.active_terminal(), None);
}

#[test]
fn closing_a_tile_in_another_workspace_removes_it_from_its_saved_layout() {
    let mut board = board();
    let first = View::workspace("m1", "w1");
    board.sync(&terminals(&["a", "b"]), |_| false);
    board.open_start("/repos/app");
    board.select(View::workspace("m1", "w2"));

    board.close(START, Some(&first));
    board.select(first);

    assert_eq!(ids(&board), ["m1:a", "m1:b"]);
}

#[test]
fn notes_tiles_and_saved_layouts_follow_a_renamed_folder() {
    let mut board = board();
    board.open_notes("m1", Some("/notes/app/plan.md"), "notes:1".into());
    board.open_notes("m2", Some("/notes/app/plan.md"), "notes:2".into());
    board.select(View::workspace("m1", "w2"));
    board.open_notes("m1", Some("/notes/app/plan.md"), "notes:3".into());

    board.follow("m1", "/notes/app", "/notes/web");

    assert_eq!(
        note(&board, "notes:3").as_deref(),
        Some("/notes/web/plan.md")
    );
    board.select(View::workspace("m1", "w1"));
    assert_eq!(
        note(&board, "notes:1").as_deref(),
        Some("/notes/web/plan.md")
    );
    assert_eq!(
        note(&board, "notes:2").as_deref(),
        Some("/notes/app/plan.md")
    );
}

#[test]
fn a_new_terminal_replaces_the_new_tile() {
    let mut board = board();
    board.sync(&terminals(&["a"]), |_| false);
    board.open_start("/repos/app");

    board.opened(select("m1", "b"));

    assert_eq!(ids(&board), ["m1:a", "m1:b"]);
    assert_eq!(board.active(), Some("m1:b"));
}

#[test]
fn keyboard_moves_swap_with_the_nearest_tile_or_take_a_new_edge() {
    let mut board = board();
    board.sync(&terminals(&["a", "b"]), |_| false);
    board.activate("m1:a");

    board.move_active(Side::Right, &board.layout().rects(AREA));
    assert_eq!(ids(&board), ["m1:b", "m1:a"]);

    board.move_active(Side::Bottom, &board.layout().rects(AREA));
    assert!((rect(&board, "m1:a").width - 900.).abs() < 0.01);
    assert_eq!(board.active(), Some("m1:a"));
}
