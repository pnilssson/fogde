use super::board::{Board, Dropped};
use super::geometry::{Rect, Side, Zone, neighbour};
use super::tile::{Dragged, Selection, Tile};

impl Board {
    pub fn can_drop(&self, dragged: &Dragged, target: Option<&str>, zone: Zone) -> bool {
        let Some(view) = &self.view else {
            return false;
        };

        match dragged {
            Dragged::Note { machine, .. } => {
                *machine == view.machine
                    && (target.is_some() || zone != Zone::Center || self.layout.is_empty())
            }
            Dragged::Terminal(selection) if selection.machine != view.machine => false,
            dragged => {
                let id = dragged.tile_id().unwrap_or_default();
                if !self.layout.has(&id) {
                    return matches!(dragged, Dragged::Terminal(_))
                        && (zone != Zone::Center || self.layout.is_empty());
                }
                match target {
                    None => zone != Zone::Center && self.layout.len() > 1,
                    Some(target) => target != id,
                }
            }
        }
    }

    pub fn drop(
        &mut self,
        dragged: &Dragged,
        target: Option<&str>,
        zone: Zone,
        key: impl FnOnce() -> String,
    ) -> Dropped {
        let placed = match dragged {
            Dragged::Note { machine, path } => {
                return self.drop_note(machine, path, target, zone, key);
            }
            Dragged::Terminal(selection) => (!self.shows(selection)).then(|| selection.clone()),
            Dragged::Tile { .. } => None,
        };
        let Some(id) = dragged.tile_id() else {
            return Dropped::Nothing;
        };

        if let Some(selection) = placed {
            self.insert(Tile::terminal(&selection), target, zone);
            self.save();
            return Dropped::Placed(selection);
        }

        match (target, zone) {
            (None, Zone::Edge(side)) => {
                if let Some(tile) = self.layout.remove(&id) {
                    self.layout.edge(side, tile);
                }
            }
            (Some(target), Zone::Center) => self.layout.swap(&id, target),
            (Some(target), Zone::Edge(side)) => {
                if let Some(tile) = self.layout.remove(&id) {
                    self.layout.beside(target, side, tile);
                }
            }
            (None, Zone::Center) => return Dropped::Nothing,
        }
        self.layout.active = Some(id);
        self.save();
        Dropped::Moved
    }

    pub(super) fn insert(&mut self, tile: Tile, target: Option<&str>, zone: Zone) {
        let id = tile.id();
        let side = match zone {
            Zone::Edge(side) => side,
            Zone::Center => Side::Right,
        };
        match target {
            Some(target) => self.layout.beside(target, side, tile),
            None => self.layout.edge(side, tile),
        }
        self.layout.active = Some(id);
    }

    pub(super) fn drop_note(
        &mut self,
        machine: &str,
        path: &str,
        target: Option<&str>,
        zone: Zone,
        key: impl FnOnce() -> String,
    ) -> Dropped {
        if let (Some(target), Zone::Center) = (target, zone) {
            match self.layout.tile(target).cloned() {
                Some(Tile::Terminal { machine, id }) => {
                    self.activate(target);
                    return Dropped::Paste(Selection::new(machine, id));
                }
                Some(Tile::Notes { .. }) => {
                    self.show_note(target, path);
                    return Dropped::Moved;
                }
                Some(Tile::Start { .. }) => {
                    self.open_notes(machine, Some(path), key());
                    return Dropped::Moved;
                }
                None => {}
            }
        }

        let tile = Tile::Notes {
            key: key(),
            machine: machine.to_owned(),
            note: Some(path.to_owned()),
        };
        self.insert(tile, target, zone);
        self.save();
        Dropped::Moved
    }

    pub fn move_active(&mut self, side: Side, rects: &[(String, Rect)]) {
        let Some(active) = self.layout.active.clone() else {
            return;
        };
        if self.layout.len() < 2 {
            return;
        }

        if let Some(other) = neighbour(rects, &active, side) {
            self.layout.swap(&active, &other);
        } else if let Some(tile) = self.layout.remove(&active) {
            self.layout.edge(side, tile);
        }
        self.layout.active = Some(active);
        self.save();
    }
}
