mod edit;

use serde::{Deserialize, Serialize};

use super::geometry::{Axis, Rect, Side};
use super::tile::Tile;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Child {
    pub node: Node,
    pub size: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "node", rename_all = "camelCase")]
pub enum Node {
    Leaf { tile: Tile },
    Split { axis: Axis, children: Vec<Child> },
}

fn leaf(tile: Tile) -> Node {
    Node::Leaf { tile }
}

impl Node {
    pub fn tiles(&self) -> Vec<&Tile> {
        match self {
            Node::Leaf { tile } => vec![tile],
            Node::Split { children, .. } => children
                .iter()
                .flat_map(|child| child.node.tiles())
                .collect(),
        }
    }

    fn path(&self, id: &str) -> Option<Vec<usize>> {
        match self {
            Node::Leaf { tile } => (tile.id() == id).then(Vec::new),
            Node::Split { children, .. } => {
                children.iter().enumerate().find_map(|(index, child)| {
                    child.node.path(id).map(|mut path| {
                        path.insert(0, index);
                        path
                    })
                })
            }
        }
    }

    fn at_mut(&mut self, path: &[usize]) -> &mut Node {
        let Some((index, rest)) = path.split_first() else {
            return self;
        };
        if !matches!(self, Node::Split { children, .. } if *index < children.len()) {
            return self;
        }

        match self {
            Node::Split { children, .. } => children[*index].node.at_mut(rest),
            node @ Node::Leaf { .. } => node,
        }
    }

    fn tile_mut(&mut self, id: &str) -> Option<&mut Tile> {
        match self {
            Node::Leaf { tile } => (tile.id() == id).then_some(tile),
            Node::Split { children, .. } => children
                .iter_mut()
                .find_map(|child| child.node.tile_mut(id)),
        }
    }

    fn without(self, id: &str) -> (Option<Node>, Option<Tile>) {
        match self {
            Node::Leaf { tile } if tile.id() == id => (None, Some(tile)),
            Node::Leaf { tile } => (Some(leaf(tile)), None),
            Node::Split { axis, children } => {
                let mut removed = None;
                let mut kept = Vec::new();
                for child in children {
                    let (node, tile) = child.node.without(id);
                    removed = removed.or(tile);
                    if let Some(node) = node {
                        kept.push(Child {
                            node,
                            size: child.size,
                        });
                    }
                }
                (normalize(axis, kept), removed)
            }
        }
    }

    pub fn rects(&self, area: Rect, out: &mut Vec<(String, Rect)>) {
        match self {
            Node::Leaf { tile } => out.push((tile.id(), area)),
            Node::Split { axis, children } => {
                let mut offset = 0.;
                for child in children {
                    let rect = match axis {
                        Axis::Row => Rect {
                            x: area.x + area.width * offset,
                            width: area.width * child.size,
                            ..area
                        },
                        Axis::Column => Rect {
                            y: area.y + area.height * offset,
                            height: area.height * child.size,
                            ..area
                        },
                    };
                    child.node.rects(rect, out);
                    offset += child.size;
                }
            }
        }
    }
}

fn normalize(axis: Axis, children: Vec<Child>) -> Option<Node> {
    let mut flat = Vec::new();
    for child in children {
        match child.node {
            Node::Split {
                axis: inner,
                children: nested,
            } if inner == axis => {
                for grandchild in nested {
                    flat.push(Child {
                        node: grandchild.node,
                        size: grandchild.size * child.size,
                    });
                }
            }
            node => flat.push(Child {
                node,
                size: child.size,
            }),
        }
    }

    let total: f32 = flat.iter().map(|child| child.size).sum();
    match flat.len() {
        0 => None,
        1 => flat.pop().map(|child| child.node),
        _ => {
            for child in &mut flat {
                child.size = if total > 0. {
                    child.size / total
                } else {
                    1. / 2.
                };
            }
            Some(Node::Split {
                axis,
                children: flat,
            })
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Layout {
    pub root: Option<Node>,
    pub active: Option<String>,
}

impl Layout {
    pub fn tiles(&self) -> Vec<&Tile> {
        self.root.as_ref().map(Node::tiles).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.tiles().len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn has(&self, id: &str) -> bool {
        self.root.as_ref().and_then(|root| root.path(id)).is_some()
    }

    pub fn tile(&self, id: &str) -> Option<&Tile> {
        self.tiles().into_iter().find(|tile| tile.id() == id)
    }

    pub fn tile_mut(&mut self, id: &str) -> Option<&mut Tile> {
        self.root.as_mut().and_then(|root| root.tile_mut(id))
    }

    pub fn active_tile(&self) -> Option<&Tile> {
        self.active.as_deref().and_then(|id| self.tile(id))
    }

    pub fn remove(&mut self, id: &str) -> Option<Tile> {
        let root = self.root.take()?;
        let (root, removed) = root.without(id);
        self.root = root;
        if removed.is_some() && self.active.as_deref() == Some(id) {
            self.active = self.tiles().first().map(|tile| tile.id());
        }
        removed
    }

    pub fn edge(&mut self, side: Side, tile: Tile) {
        let new = Child {
            node: leaf(tile),
            size: 0.5,
        };
        let Some(root) = self.root.take() else {
            self.root = Some(new.node);
            return;
        };

        let children = match root {
            Node::Split { axis, mut children } if axis == side.axis() => {
                let share = 1. / crate::num::real_usize(children.len() + 1);
                for child in &mut children {
                    child.size *= 1. - share;
                }
                let new = Child { size: share, ..new };
                if side.after() {
                    children.push(new);
                } else {
                    children.insert(0, new);
                }
                children
            }
            root => {
                let old = Child {
                    node: root,
                    size: 0.5,
                };
                if side.after() {
                    vec![old, new]
                } else {
                    vec![new, old]
                }
            }
        };
        self.root = normalize(side.axis(), children);
    }

    pub fn rects(&self, area: Rect) -> Vec<(String, Rect)> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.rects(area, &mut out);
        }
        out
    }
}
