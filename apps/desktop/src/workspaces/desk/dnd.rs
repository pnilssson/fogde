use gpui_kit::{Context, Pixels, Point, Window};

use super::Desk;
use crate::workspaces::tiles::geometry::{Rect, Side, Zone, zone};
use crate::workspaces::tiles::tile::Dragged;

const OUTER: f32 = 0.15;

#[derive(Clone, Debug)]
pub struct Hover {
    pub dragged: Dragged,
    pub target: Option<String>,
    pub zone: Zone,
    pub rect: Rect,
    pub allowed: bool,
}

impl Hover {
    pub fn note_path(&self) -> Option<String> {
        match &self.dragged {
            Dragged::Note { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    pub fn shade(&self) -> Rect {
        let rect = self.rect;
        match self.zone {
            Zone::Center => rect,
            Zone::Edge(Side::Left) => Rect {
                width: rect.width / 2.,
                ..rect
            },
            Zone::Edge(Side::Right) => Rect {
                x: rect.x + rect.width / 2.,
                width: rect.width / 2.,
                ..rect
            },
            Zone::Edge(Side::Top) => Rect {
                height: rect.height / 2.,
                ..rect
            },
            Zone::Edge(Side::Bottom) => Rect {
                y: rect.y + rect.height / 2.,
                height: rect.height / 2.,
                ..rect
            },
        }
    }
}

fn outer(area: Rect, x: f32, y: f32) -> Option<Side> {
    let distances = [
        (Side::Left, (x - area.x) / area.width),
        (Side::Right, (area.right() - x) / area.width),
        (Side::Top, (y - area.y) / area.height),
        (Side::Bottom, (area.bottom() - y) / area.height),
    ];
    distances
        .into_iter()
        .filter(|(_, distance)| *distance < OUTER && *distance >= 0.)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(side, _)| side)
}

impl Desk {
    fn area(&self) -> Option<Rect> {
        let bounds = self.bounds?;
        Some(Rect {
            x: bounds.origin.x.into(),
            y: bounds.origin.y.into(),
            width: f32::from(bounds.size.width).max(1.),
            height: f32::from(bounds.size.height).max(1.),
        })
    }

    pub(super) fn drag_move(
        &mut self,
        dragged: &Dragged,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(area) = self.area() else {
            return;
        };
        let (x, y) = (f32::from(position.x), f32::from(position.y));
        let inside = x >= area.x && x <= area.right() && y >= area.y && y <= area.bottom();
        let next = inside.then(|| {
            let rects = self.rects();
            let tile = rects.iter().find(|(_, rect)| {
                x >= rect.x && x <= rect.right() && y >= rect.y && y <= rect.bottom()
            });
            let outer = outer(area, x, y).filter(|_| self.board.layout().len() > 1);
            let (target, zone, rect) = if let Some(side) = outer {
                (None, Zone::Edge(side), area)
            } else if let Some((id, rect)) = tile {
                (Some(id.clone()), zone(*rect, x, y), *rect)
            } else {
                (None, Zone::Center, area)
            };
            let allowed = self.board.can_drop(dragged, target.as_deref(), zone);
            Hover {
                dragged: dragged.clone(),
                target,
                zone,
                rect,
                allowed,
            }
        });

        let same = match (&self.hover, &next) {
            (Some(a), Some(b)) => {
                a.target == b.target && a.zone == b.zone && a.allowed == b.allowed
            }
            (None, None) => true,
            _ => false,
        };
        self.hover = next;
        if !same {
            cx.notify();
        }
    }

    pub(super) fn drop_dragged(
        &mut self,
        dragged: &Dragged,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(hover) = self
            .hover
            .clone()
            .filter(|hover| hover.allowed && hover.dragged == *dragged)
        else {
            self.hover = None;
            cx.notify();
            return;
        };

        let mut dropped = None;
        self.edit(window, cx, |board| {
            dropped = Some(board.drop(
                dragged,
                hover.target.as_deref(),
                hover.zone,
                crate::notes::key,
            ));
        });
        if let Some(dropped) = dropped {
            self.dropped(dropped, cx);
        }
        self.hover = None;
        cx.notify();
    }

    pub(super) fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.hover.take().is_some() {
            cx.notify();
        }
    }
}
