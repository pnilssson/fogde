use super::{Child, Layout, Node, leaf};
use crate::workspaces::tiles::geometry::Side;
use crate::workspaces::tiles::tile::Tile;

impl Layout {
    pub fn beside(&mut self, target: &str, side: Side, tile: Tile) {
        let Some(path) = self.root.as_ref().and_then(|root| root.path(target)) else {
            self.edge(side, tile);
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };

        if let Some((index, parent)) = path.split_last()
            && let Node::Split { axis, children } = root.at_mut(parent)
            && *axis == side.axis()
        {
            let half = children[*index].size / 2.;
            children[*index].size = half;
            let at = if side.after() { index + 1 } else { *index };
            children.insert(
                at,
                Child {
                    node: leaf(tile),
                    size: half,
                },
            );
            return;
        }

        let node = root.at_mut(&path);
        let old = std::mem::replace(node, leaf(tile.clone()));
        let (first, second) = if side.after() {
            (old, leaf(tile))
        } else {
            (leaf(tile), old)
        };
        *node = Node::Split {
            axis: side.axis(),
            children: vec![
                Child {
                    node: first,
                    size: 0.5,
                },
                Child {
                    node: second,
                    size: 0.5,
                },
            ],
        };
    }

    pub fn swap(&mut self, a: &str, b: &str) {
        let (Some(first), Some(second)) = (self.tile(a).cloned(), self.tile(b).cloned()) else {
            return;
        };
        let Some(root) = self.root.as_mut() else {
            return;
        };
        let (Some(from), Some(to)) = (root.path(a), root.path(b)) else {
            return;
        };

        *root.at_mut(&from) = leaf(second);
        *root.at_mut(&to) = leaf(first);
    }

    pub fn resize(&mut self, path: &[usize], index: usize, delta: f32) {
        let Some(root) = self.root.as_mut() else {
            return;
        };
        if let Node::Split { children, .. } = root.at_mut(path)
            && index + 1 < children.len()
        {
            let total = children[index].size + children[index + 1].size;
            let minimum = 0.05_f32.min(total / 2.);
            let next = (children[index].size + delta).clamp(minimum, total - minimum);
            children[index].size = next;
            children[index + 1].size = total - next;
        }
    }
}
