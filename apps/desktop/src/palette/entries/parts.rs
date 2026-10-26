use super::{dispatch, item, reveal, with_desk};
use crate::assets::Glyph;
use crate::keymap::{CloseTile, MoveDown, MoveLeft, MoveRight, MoveUp};
use crate::palette::Section;
use crate::paths;
use crate::state::machines::Machine;
use crate::workspaces::desk::{self, Desk};
use crate::workspaces::selection::{Group, Waiting, view_of};
use crate::workspaces::tiles::tile::{Selection, View};
use gpui_kit::{App, Entity};
use protocol::workspaces::Workspace;
use std::rc::Rc;

pub(super) fn worktree_section(
    view: &View,
    workspace: &Workspace,
    folder: &str,
) -> Option<Section> {
    let checkout = workspace
        .checkouts
        .iter()
        .find(|checkout| checkout.path == folder && !checkout.main)?;
    let name = checkout
        .branch
        .clone()
        .unwrap_or_else(|| paths::name(&checkout.path).to_owned());
    let (gone, found) = (view.clone(), workspace.clone());
    Some(Section {
        label: name,
        items: vec![(
            item("Remove worktree", Glyph::Trash, &["remove", "delete"]),
            Rc::new(move |window, cx| {
                if let Some(desk) = desk::current(cx) {
                    crate::sidebar::remove_worktree(&gone, &found, desk, window, cx);
                }
            }),
        )],
    })
}

pub(super) fn lists(machines: &[Machine], groups: &[Group], needs: &[Waiting]) -> Vec<Section> {
    let several = machines.len() > 1;
    let mut terminals = Vec::new();
    let mut spaces = Vec::new();
    for (machine, group) in machines.iter().zip(groups) {
        for workspace in group.workspaces.unwrap_or_default() {
            let place = if several {
                format!("{} on {}", workspace.name, machine.name)
            } else {
                workspace.name.clone()
            };
            for terminal in &workspace.terminals {
                let selection = Selection::new(machine.id.to_string(), terminal.id.clone());
                if needs.iter().any(|waiting| waiting.terminal == selection) {
                    continue;
                }
                terminals.push((
                    item(
                        format!("{}  {place}", terminal.name),
                        Glyph::TerminalWindow,
                        &[&terminal.name, &workspace.name],
                    ),
                    reveal(
                        view_of(&machine.id, workspace, Some(terminal)),
                        Some(selection),
                    ),
                ));
            }
            let label = if several {
                format!("{}  {}", workspace.name, machine.name)
            } else {
                workspace.name.clone()
            };
            spaces.push((
                item(label, Glyph::Folder, &[&workspace.name, &machine.name]),
                reveal(
                    View::workspace(machine.id.to_string(), workspace.id.clone()),
                    None,
                ),
            ));
        }
    }

    [("Terminals", terminals), ("Workspaces", spaces)]
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(label, items)| Section {
            label: label.into(),
            items,
        })
        .collect()
}

pub(super) fn tile_section(desk: &Entity<Desk>, cx: &App) -> Option<Section> {
    let tiles = desk.read(cx).board().layout().len();
    if tiles == 0 {
        return None;
    }
    let mut items = vec![(
        item("Close tile", Glyph::X, &["close"]),
        dispatch(CloseTile),
    )];
    if tiles > 1 {
        items.push((
            item("Move tile left", Glyph::ArrowLeft, &["move"]),
            dispatch(MoveLeft),
        ));
        items.push((
            item("Move tile right", Glyph::ArrowRight, &["move"]),
            dispatch(MoveRight),
        ));
        items.push((
            item("Move tile up", Glyph::ArrowUp, &["move"]),
            dispatch(MoveUp),
        ));
        items.push((
            item("Move tile down", Glyph::ArrowDown, &["move"]),
            dispatch(MoveDown),
        ));
        for number in 1..=tiles.min(9) {
            items.push((
                item(format!("Go to tile {number}"), Glyph::Square, &["focus"]),
                with_desk(move |desk, window, cx| desk.focus_number(number, window, cx)),
            ));
        }
    }
    Some(Section {
        label: "Tile".into(),
        items,
    })
}
