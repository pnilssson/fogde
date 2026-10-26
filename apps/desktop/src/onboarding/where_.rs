use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, WeakEntity, Window, div, prelude::FluentBuilder,
};
use protocol::machines::AddComputer;

use super::{Onboarding, Step, parts};
use crate::assets::Glyph;
use crate::notice;
use crate::runtime;
use crate::state::machines::Machines;
use crate::state::store;
use crate::theme::{self, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Computer,
    Server,
}

pub struct Where {
    onboarding: WeakEntity<Onboarding>,
    going: Option<Place>,
}

impl Where {
    pub fn new(onboarding: WeakEntity<Onboarding>, _: &mut Context<Self>) -> Self {
        Self {
            onboarding,
            going: None,
        }
    }

    fn go(&mut self, place: Place, window: &mut Window, cx: &mut Context<Self>) {
        if self.going.is_some() {
            return;
        }
        if place == Place::Server {
            self.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Server, None, window, cx);
                })
                .ok();
            return;
        }

        self.going = Some(place);
        cx.notify();
        let own = store(cx).machines.read(cx).own().clone();
        let request = runtime::call::<AddComputer>(&own, (), cx);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.going = None;
                cx.notify();
                match result {
                    Ok(()) => {
                        let machines = store(cx).machines.clone();
                        machines.update(cx, Machines::load);
                        this.onboarding
                            .update(cx, |onboarding, cx| onboarding.computer(window, cx))
                            .ok();
                    }
                    Err(error) => notice::error("Could not add this computer", &error, cx),
                }
            })
            .ok();
        })
        .detach();
    }
}

impl Render for Where {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let onboarded = self
            .onboarding
            .upgrade()
            .is_some_and(|onboarding| onboarding.read(cx).onboarded());
        let (title, body) = if onboarded {
            (
                "Add a machine",
                "Each machine keeps its own repos, sign-ins and agents.",
            )
        } else {
            (
                "Where should your agents run?",
                "Start with one machine. You can add more anytime in Settings.",
            )
        };
        let tokens = theme::tokens(cx);
        let (surface, shadow, hover, muted) = (
            tokens.surface[2],
            tokens.shadows[1].clone(),
            tokens.hover,
            cx.theme().muted_foreground,
        );

        let places = [
            (
                Place::Computer,
                "computer",
                "This computer",
                "Agents work on the repos on this computer.",
            ),
            (
                Place::Server,
                "server",
                "A server",
                "Agents work on a Linux server you reach with SSH and keep going when this computer sleeps.",
            ),
        ];
        let cards = places.into_iter().map(|(place, id, title, description)| {
            let going = self.going == Some(place);
            h_flex()
                .id(id)
                .w_full()
                .gap_4()
                .p_4()
                .rounded_xl()
                .bg(surface)
                .shadow(shadow.clone())
                .cursor_pointer()
                .hover(|style| style.bg(hover))
                .child(
                    v_flex()
                        .flex_1()
                        .gap_1()
                        .child(div().type_title().child(title))
                        .child(div().type_body().text_color(muted).child(description)),
                )
                .child(if going {
                    Spinner::new().small().into_any_element()
                } else {
                    Icon::new(Glyph::CaretRight).small().into_any_element()
                })
                .on_click(cx.listener(move |this, _, window, cx| this.go(place, window, cx)))
        });

        parts::section(title, body)
            .child(v_flex().w_full().gap_2().children(cards))
            .when(onboarded, |page| {
                page.child(
                    Button::new("back")
                        .ghost()
                        .small()
                        .label("Back")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.onboarding
                                .update(cx, |onboarding, cx| onboarding.leave(window, cx))
                                .ok();
                        })),
                )
            })
    }
}
