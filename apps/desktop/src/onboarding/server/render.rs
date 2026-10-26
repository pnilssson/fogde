use super::{Phase, Server};
use crate::assets::Glyph;
use crate::onboarding::{Step, parts};
use crate::theme::Type;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div,
    prelude::FluentBuilder,
};

impl Render for Server {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shown = if self.problem.is_some() {
            None
        } else {
            self.phase
        };
        let chips = self.hosts.iter().map(|host| {
            let value = host.clone();
            Button::new(SharedString::from(format!("host-{host}")))
                .outline()
                .small()
                .label(host.clone())
                .on_click(cx.listener(move |this, _, window, cx| {
                    let value = value.clone();
                    this.target
                        .update(cx, |target, cx| target.set_value(value, window, cx));
                    this.problem = None;
                    cx.notify();
                }))
        });
        let back = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Where, None, window, cx);
                })
                .ok();
        }));
        let next = parts::handler(cx.listener(|this, _, window, cx| this.submit(window, cx)));

        parts::section(
            "Connect your server",
            "Sign in with your own SSH key. The server is set up for you.",
        )
        .child(
            v_flex()
                .w_full()
                .gap_2()
                .child(
                    h_flex()
                        .gap_1()
                        .child(div().type_body().child("Server"))
                        .child(
                            Button::new("what")
                                .ghost()
                                .xsmall()
                                .icon(Icon::new(Glyph::Info))
                                .tooltip("user@address, or a host from your SSH config"),
                        ),
                )
                .child(Input::new(&self.target))
                .when(!self.hosts.is_empty(), |field| {
                    field.child(h_flex().flex_wrap().gap_2().children(chips))
                })
                .when(self.invalid, |field| {
                    field.child(
                        div()
                            .type_caption()
                            .text_color(cx.theme().danger)
                            .child("Enter a server"),
                    )
                }),
        )
        .children(self.problem(cx))
        .child(parts::nav(
            Some(back),
            shown.map_or("Continue", Phase::label),
            shown.is_some(),
            self.phase.is_some(),
            next,
        ))
    }
}
