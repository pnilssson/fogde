use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};

use super::agents::{Agents, paste_button, paste_input};
use super::parts::Handler;
use crate::theme::Type;

pub fn page(
    agents: &Agents,
    account: &'static str,
    url: String,
    submitting: bool,
    cancel: Handler,
    cx: &mut Context<Agents>,
) -> AnyElement {
    let button = paste_button(submitting).on_click(cx.listener(|this, _, _, cx| this.paste(cx)));
    v_flex()
        .w_full()
        .max_w(px(384.))
        .items_center()
        .gap_8()
        .child(
            h_flex()
                .gap_1()
                .type_display()
                .child("Paste the code from")
                .child(
                    div()
                        .id("site")
                        .underline()
                        .cursor_pointer()
                        .child(account)
                        .on_click(move |_, _, cx| cx.open_url(&url)),
                ),
        )
        .child(div().w_full().child(paste_input(agents.code(), button)))
        .child(
            Button::new("cancel")
                .ghost()
                .small()
                .label("Cancel")
                .on_click(move |event, window, cx| cancel(event, window, cx)),
        )
        .into_any_element()
}
