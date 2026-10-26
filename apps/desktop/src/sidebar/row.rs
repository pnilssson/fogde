use gpui_kit::component::ActiveTheme;
use gpui_kit::component::h_flex;
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Stateful, Styled,
    div, prelude::FluentBuilder, px,
};

use crate::theme::{self, Type};

pub fn row(
    id: impl Into<ElementId>,
    icon: AnyElement,
    label: impl IntoElement,
    active: bool,
    sub: bool,
    cx: &App,
) -> Stateful<gpui_kit::Div> {
    let tokens = theme::tokens(cx);
    let hover = tokens.hover;
    h_flex()
        .id(id)
        .w_full()
        .min_w_0()
        .gap_2()
        .px_2()
        .h(if sub { px(28.) } else { px(32.) })
        .rounded_md()
        .type_body()
        .cursor_pointer()
        .text_color(cx.theme().foreground)
        .when(active, |row| row.bg(tokens.selected))
        .when(!active, |row| row.hover(|style| style.bg(hover)))
        .child(icon)
        .child(div().flex_1().min_w_0().truncate().child(label))
}
