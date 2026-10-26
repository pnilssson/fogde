use std::rc::Rc;

use gpui_kit::{App, Entity};
use protocol::notes::NoteList;

use super::recent::{matching, recent};
use crate::assets::Glyph;
use crate::palette::entries::item;
use crate::palette::{Run, Section};
use crate::paths;
use crate::persist;
use crate::state::store;
use crate::workspaces::desk::{self, Desk};

pub struct Source {
    machine: String,
    list: NoteList,
    viewed: Vec<String>,
}

pub fn source(desk: &Entity<Desk>, cx: &App) -> Option<Source> {
    let board = desk.read(cx);
    let view = board.view()?;
    let workspace = board.workspace(cx)?;
    if workspace.missing {
        return None;
    }
    let list = store(cx).notes.read(cx).list(&view.machine)?.clone();
    let viewed = persist::saved(cx)
        .recent
        .get(&view.machine)
        .cloned()
        .unwrap_or_default();
    Some(Source {
        machine: view.machine.clone(),
        list,
        viewed,
    })
}

fn open(machine: String, path: String) -> Run {
    Rc::new(move |window, cx| {
        if let Some(desk) = desk::current(cx) {
            let (machine, path) = (machine.clone(), path.clone());
            desk.update(cx, |desk, cx| {
                desk.edit(window, cx, |board| {
                    board.open_notes(&machine, Some(&path), super::key());
                });
            });
        }
    })
}

pub fn section(source: &Source, query: &str) -> Option<Section> {
    if source.list.notes.is_empty() {
        return None;
    }
    let searching = !query.trim().is_empty();
    let notes = if searching {
        matching(&source.list, query)
    } else {
        recent(&source.list.notes, &source.viewed)
    };
    let items = notes
        .into_iter()
        .map(|note| {
            let folder = paths::relative(&note.folder, &source.list.root).unwrap_or_default();
            let label = if folder.is_empty() {
                note.name.clone()
            } else {
                format!("{}  {folder}", note.name)
            };
            (
                item(label, Glyph::FileText, &[&note.name, &folder]),
                open(source.machine.clone(), note.path.clone()),
            )
        })
        .collect();
    Some(Section {
        label: if searching {
            "Notes".into()
        } else {
            "Recent notes".into()
        },
        items,
    })
}
