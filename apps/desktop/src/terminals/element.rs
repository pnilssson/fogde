use gpui_kit::{
    App, BorderStyle, Bounds, DispatchPhase, ElementInputHandler, Entity, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, ScrollEvent, Style, TextAlign, Window, fill, outline, point, px,
    relative, size,
};
use terminal::grid::vte::ansi::CursorShape;

use super::layout::{self, Layout};
use super::view::TerminalView;
use super::{access, links, select};

pub struct TerminalElement {
    view: Entity<TerminalView>,
    access: Option<access::Rows>,
}

impl TerminalElement {
    pub fn new(view: Entity<TerminalView>, access: Option<access::Rows>) -> Self {
        Self { view, access }
    }
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl gpui_kit::Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = Layout;

    fn id(&self) -> Option<gpui_kit::ElementId> {
        Some("terminal-grid".into())
    }

    fn a11y_role(&self) -> Option<gpui_kit::Role> {
        self.access.as_ref().map(|_| gpui_kit::Role::Terminal)
    }

    fn write_a11y_info(&self, node: &mut gpui_kit::accesskit::Node) {
        if let Some(rows) = &self.access {
            access::describe(rows, node);
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        _: &mut Self::PrepaintState,
        builder: &mut gpui_kit::A11ySubtreeBuilder,
    ) {
        if let Some(rows) = &self.access {
            access::children(rows, builder);
        }
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (): &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let cell = layout::measure(window);
        let cols = crate::num::floor_u16(bounds.size.width / cell.width);
        let rows = crate::num::floor_u16(bounds.size.height / cell.height);
        let measured = (cols >= 1 && rows >= 1).then_some(protocol::terminals::Size { cols, rows });

        self.view.update(cx, |view, cx| {
            view.cell = Some(cell);
            view.bounds = Some(bounds);
            if view.measured_size() != measured {
                let view_entity = cx.entity();
                cx.defer(move |cx| {
                    view_entity.update(cx, |view, cx| view.measured(measured, cx));
                });
            }
        });

        let view = self.view.read(cx);
        layout::layout(view, bounds.origin, cell, window)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (): &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (focus, selection, cursor_color, accent) = {
            let view = self.view.read(cx);
            (
                view.focus.clone(),
                view.palette.selection,
                view.palette.cursor,
                view.palette.accent,
            )
        };
        let focused = focus.is_focused(window);

        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );
        self.listen(bounds, window);

        window.paint_layer(bounds, |window| {
            for (bounds, color) in &layout.backgrounds {
                window.paint_quad(fill(*bounds, *color));
            }
            for bounds in &layout.selection {
                window.paint_quad(fill(*bounds, selection));
            }
            for glyphs in &layout.glyphs {
                let _ = glyphs.line.paint(
                    glyphs.origin,
                    layout::LINE_HEIGHT,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
            layout::paint_drawn(&layout.drawn, window);

            if let Some(cursor) = &layout.cursor {
                let cell = cursor.bounds;
                let beam = Bounds::new(cell.origin, size(px(2.), cell.size.height));
                let under = Bounds::new(
                    point(cell.origin.x, cell.origin.y + cell.size.height - px(2.)),
                    size(cell.size.width, px(2.)),
                );
                if !focused || cursor.shape == CursorShape::HollowBlock {
                    window.paint_quad(outline(cell, cursor_color, BorderStyle::Solid));
                } else {
                    match cursor.shape {
                        CursorShape::Block => {
                            window.paint_quad(fill(cell, cursor_color));
                            let _ = accent;
                        }
                        CursorShape::Underline => window.paint_quad(fill(under, cursor_color)),
                        _ => window.paint_quad(fill(beam, cursor_color)),
                    }
                }
            }
        });
    }
}

impl TerminalElement {
    fn listen(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        let view = self.view.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || event.button != MouseButton::Left
                || !bounds.contains(&event.position)
            {
                return;
            }
            view.update(cx, |view, cx| {
                window.focus(&view.focus, cx);
                let link = event.modifiers.secondary(cx);
                if link && let Some(url) = links::at(view, event.position) {
                    links::open(&url, cx);
                    return;
                }
                select::start(view, event.position, event.click_count);
                cx.notify();
            });
            cx.stop_propagation();
        });

        let view = self.view.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
            if phase != DispatchPhase::Bubble || !event.dragging() {
                return;
            }
            view.update(cx, |view, cx| {
                if view.selecting {
                    select::extend(view, event.position);
                    cx.notify();
                }
            });
        });

        let view = self.view.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                return;
            }
            view.update(cx, |view, cx| {
                if view.selecting {
                    view.selecting = false;
                    select::copy(view, cx);
                }
            });
        });

        let view = self.view.clone();
        window.on_mouse_event(move |event: &ScrollEvent, phase, _, cx| {
            if phase != DispatchPhase::Bubble || !bounds.contains(&event.position) {
                return;
            }
            view.update(cx, |view, _| select::wheel(view, event));
            cx.stop_propagation();
        });
    }
}
