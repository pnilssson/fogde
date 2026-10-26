#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use super::geometry::Side;
use super::tile::{START, Selection, Tile, View};
use super::tree::Layout;

#[derive(Debug, PartialEq, Eq)]
pub enum Dropped {
    Nothing,
    Moved,
    Placed(Selection),
    Paste(Selection),
}

#[derive(Default)]
pub struct Board {
    pub(super) view: Option<View>,
    pub(super) layout: Layout,
    pub(super) saved: BTreeMap<String, Layout>,
    pub(super) closed: BTreeSet<String>,
    pending: Option<Selection>,
}

impl Board {
    pub fn new(saved: BTreeMap<String, Layout>, closed: BTreeSet<String>) -> Self {
        Self {
            saved,
            closed,
            ..Self::default()
        }
    }

    pub fn view(&self) -> Option<&View> {
        self.view.as_ref()
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn saved(&self) -> &BTreeMap<String, Layout> {
        &self.saved
    }

    pub fn closed(&self) -> &BTreeSet<String> {
        &self.closed
    }

    pub(super) fn here(&self, view: &View) -> bool {
        self.view.as_ref() == Some(view)
    }

    pub(super) fn save(&mut self) {
        if let Some(view) = &self.view {
            self.saved.insert(view.key(), self.layout.clone());
        }
    }

    pub fn change(&mut self, edit: impl FnOnce(&mut Layout)) {
        edit(&mut self.layout);
        self.save();
    }

    pub fn select(&mut self, next: View) -> bool {
        self.set_open(&next.home(), true);
        if self.here(&next) {
            return false;
        }

        self.layout = self.saved.get(&next.key()).cloned().unwrap_or_default();
        self.view = Some(next);
        true
    }

    pub fn leave(&mut self, machine: &str, id: &str) {
        if self
            .view
            .as_ref()
            .is_some_and(|view| view.machine == machine && view.id == id)
        {
            self.view = None;
            self.layout = Layout::default();
        }
    }

    pub fn activate(&mut self, id: &str) {
        if self.layout.has(id) && self.layout.active.as_deref() != Some(id) {
            self.change(|layout| layout.active = Some(id.to_owned()));
        }
    }

    pub fn active(&self) -> Option<&str> {
        self.layout.active.as_deref()
    }

    pub fn active_terminal(&self) -> Option<Selection> {
        self.layout.active_tile().and_then(Tile::selection)
    }

    pub fn shows(&self, selection: &Selection) -> bool {
        self.layout.has(&selection.tile())
    }

    pub fn show(&mut self, selection: Selection) {
        if self.shows(&selection) {
            self.activate(&selection.tile());
            return;
        }
        self.pending = Some(selection);
    }

    pub fn sync(&mut self, terminals: &[Selection], keep: impl Fn(&Selection) -> bool) {
        let gone: Vec<String> = self
            .layout
            .tiles()
            .into_iter()
            .filter_map(Tile::selection)
            .filter(|selection| !terminals.contains(selection) && !keep(selection))
            .map(|selection| selection.tile())
            .collect();
        let missing: Vec<&Selection> = terminals
            .iter()
            .filter(|terminal| !self.shows(terminal))
            .collect();
        if gone.is_empty() && missing.is_empty() && self.pending.is_none() {
            return;
        }

        for id in gone {
            self.layout.remove(&id);
        }
        for terminal in missing {
            self.layout.edge(Side::Right, Tile::terminal(terminal));
            if self.layout.active.is_none() {
                self.layout.active = Some(terminal.tile());
            }
        }
        if let Some(pending) = self
            .pending
            .take_if(|pending| self.layout.has(&pending.tile()))
        {
            self.layout.active = Some(pending.tile());
        }
        self.save();
    }

    pub(super) fn beside_active(&mut self, tile: Tile) {
        let id = tile.id();
        match self.layout.active.clone() {
            Some(active) => self.layout.beside(&active, Side::Right, tile),
            None => self.layout.edge(Side::Right, tile),
        }
        self.layout.active = Some(id);
    }

    pub fn open_start(&mut self, folder: &str) {
        if let Some(Tile::Start { folder: current }) = self.layout.tile_mut(START) {
            folder.clone_into(current);
        } else {
            self.beside_active(Tile::Start {
                folder: folder.to_owned(),
            });
        }
        self.layout.active = Some(START.to_owned());
        self.save();
    }

    pub(super) fn replace_start(&mut self, tile: Tile) -> bool {
        let id = tile.id();
        if !self.layout.has(START) {
            return false;
        }

        self.layout.remove(&id);
        if let Some(start) = self.layout.tile_mut(START) {
            *start = tile;
        }
        self.layout.active = Some(id);
        true
    }

    pub fn opened(&mut self, selection: Selection) {
        if !self.replace_start(Tile::terminal(&selection)) {
            self.show(selection);
            return;
        }
        self.save();
    }

    pub fn open_notes(&mut self, machine: &str, note: Option<&str>, key: String) {
        let tile = Tile::Notes {
            key,
            machine: machine.to_owned(),
            note: note.map(str::to_owned),
        };
        if !self.replace_start(tile.clone()) {
            self.beside_active(tile);
        }
        self.save();
    }

    pub fn show_note(&mut self, id: &str, path: &str) {
        if let Some(Tile::Notes { note, .. }) = self.layout.tile_mut(id) {
            *note = Some(path.to_owned());
        }
        self.layout.active = Some(id.to_owned());
        self.save();
    }

    pub fn close(&mut self, id: &str, view: Option<&View>) {
        match view {
            Some(view) if !self.here(view) => {
                if let Some(layout) = self.saved.get_mut(&view.key()) {
                    layout.remove(id);
                }
            }
            _ => {
                self.layout.remove(id);
                self.save();
            }
        }
    }
}
