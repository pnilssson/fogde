use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Axis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl Side {
    pub fn axis(self) -> Axis {
        match self {
            Side::Left | Side::Right => Axis::Row,
            Side::Top | Side::Bottom => Axis::Column,
        }
    }

    pub fn after(self) -> bool {
        matches!(self, Side::Right | Side::Bottom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Center,
    Edge(Side),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
}

const EDGE: f32 = 0.15;

pub fn zone(rect: Rect, x: f32, y: f32) -> Zone {
    let distances = [
        (Side::Left, (x - rect.x) / rect.width),
        (Side::Right, (rect.right() - x) / rect.width),
        (Side::Top, (y - rect.y) / rect.height),
        (Side::Bottom, (rect.bottom() - y) / rect.height),
    ];
    distances
        .into_iter()
        .filter(|(_, distance)| *distance < EDGE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(Zone::Center, |(side, _)| Zone::Edge(side))
}

fn gap(from: Rect, to: Rect, side: Side) -> f32 {
    match side {
        Side::Left => from.x - to.right(),
        Side::Right => to.x - from.right(),
        Side::Top => from.y - to.bottom(),
        Side::Bottom => to.y - from.bottom(),
    }
}

fn overlap(from: Rect, to: Rect, side: Side) -> f32 {
    match side.axis() {
        Axis::Row => from.bottom().min(to.bottom()) - from.y.max(to.y),
        Axis::Column => from.right().min(to.right()) - from.x.max(to.x),
    }
}

pub fn neighbour(rects: &[(String, Rect)], home: &str, side: Side) -> Option<String> {
    let from = rects.iter().find(|(id, _)| id == home)?.1;
    let mut best: Option<(&String, f32, f32)> = None;

    for (id, to) in rects {
        if id == home {
            continue;
        }
        let near = (gap(from, *to, side), overlap(from, *to, side));
        if near.0 < -1. || near.1 <= 0. {
            continue;
        }
        let better = best.is_none_or(|(_, gap, overlap)| {
            near.0
                .total_cmp(&gap)
                .then(overlap.total_cmp(&near.1))
                .is_lt()
        });
        if better {
            best = Some((id, near.0, near.1));
        }
    }

    best.map(|(id, ..)| id.clone())
}

pub fn numbered(rects: &[(String, Rect)]) -> Vec<String> {
    let mut placed: Vec<&(String, Rect)> = rects.iter().collect();
    placed.sort_by(|a, b| {
        a.1.x
            .round()
            .total_cmp(&b.1.x.round())
            .then(a.1.y.round().total_cmp(&b.1.y.round()))
    });
    placed.into_iter().map(|(id, _)| id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn grid() -> Vec<(String, Rect)> {
        vec![
            ("a".into(), rect(0., 0., 450., 300.)),
            ("b".into(), rect(0., 300., 450., 300.)),
            ("c".into(), rect(450., 0., 450., 600.)),
        ]
    }

    #[test]
    fn the_neighbour_is_the_nearest_tile_that_overlaps_on_that_side() {
        let rects = grid();
        assert_eq!(neighbour(&rects, "a", Side::Right).as_deref(), Some("c"));
        assert_eq!(neighbour(&rects, "a", Side::Bottom).as_deref(), Some("b"));
        assert_eq!(neighbour(&rects, "c", Side::Left).as_deref(), Some("a"));
        assert_eq!(neighbour(&rects, "a", Side::Left), None);
        assert_eq!(neighbour(&rects, "a", Side::Top), None);
    }

    #[test]
    fn tiles_are_numbered_left_to_right_then_top_to_bottom() {
        assert_eq!(numbered(&grid()), ["a", "b", "c"]);
    }

    #[test]
    fn the_outer_fifteen_percent_of_a_tile_is_an_edge() {
        let tile = rect(0., 0., 100., 100.);
        assert_eq!(zone(tile, 50., 50.), Zone::Center);
        assert_eq!(zone(tile, 5., 50.), Zone::Edge(Side::Left));
        assert_eq!(zone(tile, 50., 95.), Zone::Edge(Side::Bottom));
    }
}
