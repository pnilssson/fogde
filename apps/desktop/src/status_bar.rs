use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex};
use gpui_kit::{AnyElement, App, Entity, IntoElement, ParentElement, Styled, div};

use crate::assets::Glyph;
use crate::settings;
use crate::theme::Type;
use crate::usage::Meters;

pub fn render(meters: &Entity<Meters>, cx: &App) -> AnyElement {
    h_flex()
        .flex_shrink_0()
        .h_10()
        .gap_4()
        .px_2()
        .border_t_1()
        .border_color(cx.theme().border)
        .type_caption()
        .child(
            Button::new("settings")
                .ghost()
                .small()
                .icon(Icon::new(Glyph::GearSix))
                .tooltip("Settings")
                .on_click(|_, window, cx| settings::open(window, cx)),
        )
        .child(div().flex_1())
        .child(meters.clone())
        .into_any_element()
}
