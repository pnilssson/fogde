use super::*;
use crate::workspaces::tiles::board::Dropped;
use crate::workspaces::tiles::geometry::{Side, Zone};
use crate::workspaces::tiles::tile::{Dragged, START, Tile};

#[test]
fn a_note_dropped_on_a_terminal_pastes_its_path_there() {
    let mut board = board();
    board.sync(&terminals(&["a"]), |_| false);
    let note = Dragged::Note {
        machine: "m1".into(),
        path: "/notes/plan.md".into(),
    };

    let dropped = board.drop(&note, Some("m1:a"), Zone::Center, key);

    assert_eq!(dropped, Dropped::Paste(select("m1", "a")));
    assert_eq!(ids(&board), ["m1:a"]);
}

#[test]
fn a_note_dropped_on_an_edge_opens_in_a_new_tile_beside_it() {
    let mut board = board();
    board.sync(&terminals(&["a"]), |_| false);
    let note = Dragged::Note {
        machine: "m1".into(),
        path: "/notes/plan.md".into(),
    };

    board.drop(&note, Some("m1:a"), Zone::Edge(Side::Left), || {
        "notes:1".into()
    });

    assert_eq!(ids(&board), ["notes:1", "m1:a"]);
}

#[test]
fn a_note_dropped_on_a_notes_tile_or_the_new_tile_shows_it_there() {
    let mut board = board();
    board.open_start("/repos/app");
    let plan = Dragged::Note {
        machine: "m1".into(),
        path: "/notes/plan.md".into(),
    };
    board.drop(&plan, Some(START), Zone::Center, || "notes:1".into());
    assert_eq!(ids(&board), ["notes:1"]);

    let backlog = Dragged::Note {
        machine: "m1".into(),
        path: "/notes/backlog.md".into(),
    };
    board.drop(&backlog, Some("notes:1"), Zone::Center, key);

    assert_eq!(
        board.layout().tile("notes:1"),
        Some(&Tile::Notes {
            key: "notes:1".into(),
            machine: "m1".into(),
            note: Some("/notes/backlog.md".into()),
        })
    );
}

#[test]
fn a_note_drops_only_into_a_workspace_on_its_own_machine() {
    let mut board = board();
    board.sync(&terminals(&["a"]), |_| false);
    let mine = Dragged::Note {
        machine: "m1".into(),
        path: "/notes/plan.md".into(),
    };
    let theirs = Dragged::Note {
        machine: "m2".into(),
        path: "/notes/plan.md".into(),
    };

    assert!(board.can_drop(&mine, Some("m1:a"), Zone::Center));
    assert!(!board.can_drop(&theirs, Some("m1:a"), Zone::Center));
}
