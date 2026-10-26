use gpui_kit::{
    AnyElement, AppContext, Context, CursorStyle, DragMoveEvent, ElementId, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px, relative,
};

use super::Desk;
use crate::theme;
use crate::workspaces::tiles::geometry::Axis;
use crate::workspaces::tiles::tree::Node;

#[derive(Clone)]
pub struct Resize {
    path: Vec<usize>,
    index: usize,
}

pub struct Blank;

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

fn key(prefix: &str, path: &[usize]) -> ElementId {
    let parts: Vec<String> = path.iter().map(ToString::to_string).collect();
    ElementId::Name(format!("{prefix}-{}", parts.join("-")).into())
}

impl Desk {
    pub(super) fn node(
        &self,
        node: &Node,
        path: &[usize],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (axis, children) = match node {
            Node::Leaf { tile } => return self.tile(tile, window, cx),
            Node::Split { axis, children } => (*axis, children),
        };
        let row = axis == Axis::Row;
        let sizes: Vec<f32> = children.iter().map(|child| child.size).collect();
        let border = theme::tokens(cx).border;
        let owner = path.to_vec();

        let mut container = div()
            .id(key("split", path))
            .flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .when(row, Styled::flex_row)
            .when(!row, Styled::flex_col)
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<Resize>, _, cx| {
                    let resize = event.drag(cx).clone();
                    if resize.path != owner {
                        return;
                    }
                    let bounds = event.bounds;
                    let position = event.event.position;
                    let fraction = if row {
                        (position.x - bounds.origin.x) / bounds.size.width
                    } else {
                        (position.y - bounds.origin.y) / bounds.size.height
                    };
                    let before: f32 = sizes[..resize.index].iter().sum();
                    let delta = fraction - before - sizes[resize.index];
                    this.board
                        .change(|layout| layout.resize(&resize.path, resize.index, delta));
                    this.changed(cx);
                }),
            );

        for (index, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            if index > 0 {
                let resize = Resize {
                    path: path.to_vec(),
                    index: index - 1,
                };
                container = container.child(
                    div()
                        .relative()
                        .flex_none()
                        .bg(border)
                        .when(row, |line| line.w(px(1.)).h_full())
                        .when(!row, |line| line.h(px(1.)).w_full())
                        .child(
                            div()
                                .id(key("divider", &child_path))
                                .absolute()
                                .when(row, |hit| {
                                    hit.top_0()
                                        .bottom_0()
                                        .left(px(-4.))
                                        .w(px(9.))
                                        .cursor(CursorStyle::ResizeLeftRight)
                                })
                                .when(!row, |hit| {
                                    hit.left_0()
                                        .right_0()
                                        .top(px(-4.))
                                        .h(px(9.))
                                        .cursor(CursorStyle::ResizeUpDown)
                                })
                                .on_drag(resize, |_, _, _, cx| cx.new(|_| Blank)),
                        ),
                );
            }
            container = container.child(
                div()
                    .flex()
                    .flex_basis(relative(child.size))
                    .flex_shrink(1.)
                    .min_w_0()
                    .min_h_0()
                    .child(self.node(&child.node, &child_path, window, cx)),
            );
        }

        container.into_any_element()
    }
}
