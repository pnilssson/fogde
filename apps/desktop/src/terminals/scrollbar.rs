use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Bounds, Context, DispatchPhase, Hitbox, HitboxBehavior, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Styled, canvas, div, fill,
    point, px, size,
};

use super::outgoing::Outgoing;
use super::view::TerminalView;
use crate::theme;

const MINIMUM: Pixels = px(32.);

fn thumb(track: Bounds<Pixels>, share: f32, place: f32) -> Bounds<Pixels> {
    let height = (track.size.height * share)
        .max(MINIMUM)
        .min(track.size.height);
    let top = (track.size.height - height) * place;
    Bounds::new(
        point(track.origin.x, track.origin.y + top),
        size(track.size.width, height),
    )
}

impl TerminalView {
    pub(super) fn scrollbar(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let scroll = self.scroll.filter(|scroll| scroll.max > 0)?;
        let view = cx.entity();
        let tokens = &theme::tokens(cx).terminal;
        let colors = (
            tokens.scrollbar,
            tokens.scrollbar_hover,
            tokens.scrollbar_active,
        );
        let radius = cx.theme().radius * 0.6;
        let offset = self.seeking.unwrap_or(scroll.offset);
        let dragging = self.grab.is_some();

        Some(
            div().absolute().top_2().bottom_2().right_0().w_2().child(
                canvas(
                    |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                    move |track: Bounds<Pixels>, hitbox: Hitbox, window, _| {
                        let thumb = thumb(track, scroll.share(), scroll.place(offset));
                        let hovered =
                            hitbox.is_hovered(window) && thumb.contains(&window.mouse_position());
                        let color = if dragging {
                            colors.2
                        } else if hovered {
                            colors.1
                        } else {
                            colors.0
                        };
                        window.paint_quad(fill(thumb, color).corner_radii(radius));

                        let grabbing = view.clone();
                        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                            if phase != DispatchPhase::Bubble
                                || event.button != MouseButton::Left
                                || !hitbox.is_hovered(window)
                                || !thumb.contains(&event.position)
                            {
                                return;
                            }
                            grabbing.update(cx, |view, cx| {
                                view.grab = Some((
                                    event.position.y,
                                    offset,
                                    track.size.height - thumb.size.height,
                                ));
                                cx.notify();
                            });
                            cx.stop_propagation();
                        });

                        let moving = view.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble {
                                return;
                            }
                            moving.update(cx, |view, cx| {
                                let Some((start, from, travel)) = view.grab else {
                                    return;
                                };
                                let next = scroll.dragged(
                                    from,
                                    (event.position.y - start).into(),
                                    travel.into(),
                                );
                                if view.seeking != Some(next) {
                                    view.seeking = Some(next);
                                    view.send(Outgoing::Seek(next));
                                    cx.notify();
                                }
                            });
                        });

                        let releasing = view.clone();
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble {
                                return;
                            }
                            releasing.update(cx, |view, cx| {
                                if view.grab.take().is_some() {
                                    view.seeking = None;
                                    cx.notify();
                                }
                            });
                        });
                    },
                )
                .size_full(),
            ),
        )
    }
}
