mod machines;
mod notes;
mod panel;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Icon, Sizable, WindowExt, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext, Div, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::FluentBuilder, px,
};

use crate::assets::Glyph;
use crate::theme::Type;

pub fn open(window: &mut Window, cx: &mut App) {
    let panel = cx.new(|cx| panel::Panel::new(window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title("Settings").w(px(560.)).child(panel.clone())
    });
}

fn section(title: &'static str, hint: Option<&'static str>, action: Option<AnyElement>) -> Div {
    v_flex().gap_1().child(
        h_flex()
            .h_8()
            .gap_2()
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(div().type_title().truncate().child(title))
                    .when_some(hint, |row, hint| {
                        row.child(
                            Button::new(SharedString::from(format!("hint-{title}")))
                                .ghost()
                                .xsmall()
                                .icon(Icon::new(Glyph::Info))
                                .tooltip(hint),
                        )
                    }),
            )
            .children(action),
    )
}

fn row(label: impl Into<SharedString>, control: impl IntoElement) -> Div {
    h_flex()
        .h_8()
        .gap_2()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .type_body()
                .child(label.into()),
        )
        .child(control)
}
