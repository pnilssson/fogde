pub mod desk;
pub mod dialogs;
pub mod order;
pub mod selection;
pub mod start;
pub mod status;
pub mod tile_bar;
pub mod tiles;

use gpui_kit::component::{ActiveTheme, Icon, Sizable};
use gpui_kit::{
    AnyElement, App, InteractiveElement, IntoElement, StatefulInteractiveElement, Styled, div, img,
    px,
};
use protocol::agents::Agent;
use protocol::terminals::Status;

use crate::assets::Glyph;
use crate::theme;

pub fn status_dot(status: Status, cx: &App) -> Option<AnyElement> {
    let tokens = theme::tokens(cx);
    let label = status::label(status)?;
    let color = match status {
        Status::Idle | Status::Working => tokens.warning,
        Status::Blocked => tokens.destructive,
        Status::Done => tokens.success,
    };

    Some(
        div()
            .id(label)
            .role(gpui_kit::Role::Image)
            .aria_label(label)
            .flex_shrink_0()
            .size_2()
            .rounded_full()
            .bg(color)
            .into_any_element(),
    )
}

pub fn agent_icon(agent: Agent, cx: &App) -> AnyElement {
    match agent {
        Agent::Claude => img("fogde/brands/claude-spark.svg")
            .flex_shrink_0()
            .size(px(16.))
            .into_any_element(),
        Agent::Codex => Icon::default()
            .path("fogde/brands/openai.svg")
            .small()
            .text_color(cx.theme().foreground)
            .into_any_element(),
        Agent::Copilot => Icon::default()
            .path("fogde/brands/copilot.svg")
            .small()
            .text_color(cx.theme().foreground)
            .into_any_element(),
    }
}

pub fn terminal_icon(agent: Option<Agent>, cx: &App) -> AnyElement {
    match agent {
        Some(agent) => agent_icon(agent, cx),
        None => Icon::new(Glyph::TerminalWindow).small().into_any_element(),
    }
}
