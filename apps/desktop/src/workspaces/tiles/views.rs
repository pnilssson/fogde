use super::board::Board;
use super::tile::{START, Tile, View, note_title};
use super::tree::Layout;
use crate::paths;

impl Board {
    pub(super) fn layout_of(&self, view: &View) -> Option<&Layout> {
        if self.here(view) {
            return Some(&self.layout);
        }
        self.saved.get(&view.key())
    }

    pub fn keep_machines(&mut self, machines: &[&str]) {
        let known = |key: &str| {
            machines
                .iter()
                .any(|machine| key.starts_with(&format!("{machine}/")))
        };
        self.closed.retain(|key| known(key));
        self.saved.retain(|key, _| known(key));
        if self
            .view
            .as_ref()
            .is_some_and(|view| !machines.contains(&view.machine.as_str()))
        {
            self.view = None;
            self.layout = Layout::default();
        }
    }

    pub fn forget(&mut self, view: &View) {
        let gone = view.key();
        let kept = |key: &str| key != gone && !key.starts_with(&format!("{gone}/"));
        self.closed.retain(|key| kept(key));
        self.saved.retain(|key, _| kept(key));
    }

    pub fn follow(&mut self, machine: &str, from: &str, to: &str) {
        let follow = |layout: &mut Layout| {
            let ids: Vec<String> = layout.tiles().into_iter().map(Tile::id).collect();
            for id in ids {
                if let Some(Tile::Notes {
                    machine: owner,
                    note: Some(note),
                    ..
                }) = layout.tile_mut(&id)
                    && owner == machine
                    && let Some(moved) = crate::notes::renames::moved(note, from, to)
                {
                    *note = moved;
                }
            }
        };
        follow(&mut self.layout);
        for layout in self.saved.values_mut() {
            follow(layout);
        }
        self.save();
    }

    pub fn is_open(&self, view: &View) -> bool {
        !self.closed.contains(&view.key())
    }

    pub fn set_open(&mut self, view: &View, open: bool) {
        let key = view.key();
        if open {
            self.closed.remove(&key);
        } else {
            self.closed.insert(key);
        }
    }

    pub fn shared_start(&self, view: &View) -> Option<String> {
        let layout = self.layout_of(view)?;
        if layout.len() < 2 {
            return None;
        }
        match layout.tile(START) {
            Some(Tile::Start { folder }) => Some(folder.clone()),
            _ => None,
        }
    }

    pub fn note_tiles(&self, view: &View) -> Vec<(String, String)> {
        self.layout_of(view)
            .map(|layout| {
                layout
                    .tiles()
                    .into_iter()
                    .filter_map(|tile| match tile {
                        Tile::Notes { note, .. } => Some((tile.id(), note_title(note.as_deref()))),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn place_label(folder: &str, home: &str) -> Option<String> {
        let label = paths::relative(folder, home).unwrap_or_else(|| paths::name(folder).to_owned());
        (!label.is_empty()).then_some(label)
    }
}
