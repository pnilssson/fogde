mod machine;
mod new_worktree;
mod row;
mod rows;
mod workspace;

pub use new_worktree::open as new_worktree;
pub use workspace::remove as remove_worktree;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::{ActiveTheme, Icon, v_flex};
use gpui_kit::{
    AnyElement, App, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Subscription, Window, div,
};

use crate::assets::Glyph;
use crate::keymap::{self, OpenPalette};
use crate::state::store;
use crate::updates;
use crate::workspaces::desk::Desk;
use crate::workspaces::order::Edge;

pub struct Sidebar {
    desk: Entity<Desk>,
    over: Option<(String, Edge)>,
    _subscriptions: Vec<Subscription>,
}

impl Sidebar {
    pub fn new(desk: Entity<Desk>, cx: &mut Context<Self>) -> Self {
        let state = store(cx);
        let (machines, workspaces, agents) = (
            state.machines.clone(),
            state.workspaces.clone(),
            state.agents.clone(),
        );
        let subscriptions = vec![
            cx.observe(&desk, |_, _, cx| cx.notify()),
            cx.observe(&machines, |_, _, cx| cx.notify()),
            cx.observe(&workspaces, |_, _, cx| cx.notify()),
            cx.observe(&agents, |_, _, cx| cx.notify()),
        ];
        let mut subscriptions = subscriptions;
        if let Some(updates) = updates::entity(cx) {
            subscriptions.push(cx.observe(&updates, |_, _, cx| cx.notify()));
        }
        Self {
            desk,
            over: None,
            _subscriptions: subscriptions,
        }
    }

    fn search(window: &mut Window, cx: &App) -> AnyElement {
        Button::new("search")
            .secondary()
            .w_full()
            .justify_start()
            .text_color(cx.theme().muted_foreground)
            .icon(Icon::new(Glyph::MagnifyingGlass))
            .label("Search")
            .child(div().flex_1())
            .children(Kbd::binding_for_action(&OpenPalette, None, window))
            .on_click(|_, window, cx| keymap::open_palette(window, cx))
            .into_any_element()
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let machines: Vec<_> = store(cx).machines.read(cx).connected().cloned().collect();
        let groups: Vec<AnyElement> = machines
            .iter()
            .map(|machine| self.machine(machine, window, cx))
            .collect();

        v_flex()
            .size_full()
            .min_h_0()
            .child(div().p_2().child(Self::search(window, cx)))
            .child(
                v_flex()
                    .id("sidebar-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_2()
                    .gap_2()
                    .children(groups),
            )
            .children(updates::button(cx))
    }
}
