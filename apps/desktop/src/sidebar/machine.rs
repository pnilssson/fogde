use client::State;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled,
    Window, div,
};

use super::Sidebar;
use crate::assets::Glyph;
use crate::state::cache::Cache;
use crate::state::machines::Machine;
use crate::state::store;
use crate::theme::Type;
use crate::workspaces::dialogs;

impl Sidebar {
    fn action(
        machine: &Machine,
        down: bool,
        loading: bool,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = machine.id.clone();
        if down {
            let disconnected = store(cx).machines.read(cx).state(&machine.id) != State::Connected;
            let label = error.unwrap_or_else(|| {
                if disconnected {
                    "Reconnect".into()
                } else {
                    "Retry".into()
                }
            });
            let retrying = store(cx)
                .workspaces
                .read(cx)
                .cache(&machine.id)
                .is_some_and(Cache::loading);
            return Button::new(SharedString::from(format!("recover-{id}")))
                .ghost()
                .small()
                .loading(retrying && !disconnected)
                .disabled(retrying && !disconnected)
                .icon(Icon::new(Glyph::ArrowClockwise))
                .tooltip(label)
                .on_click(move |_, _, cx| {
                    let state = store(cx);
                    if state.machines.read(cx).state(&id) == State::Connected {
                        state
                            .workspaces
                            .clone()
                            .update(cx, |workspaces, cx| workspaces.reload(&id, cx));
                    } else {
                        state
                            .machines
                            .clone()
                            .update(cx, |machines, cx| machines.reconnect(&id, cx));
                    }
                })
                .into_any_element();
        }
        if loading {
            return div()
                .px_2()
                .child(Spinner::new().small())
                .into_any_element();
        }

        let machine = machine.clone();
        Button::new(SharedString::from(format!("new-workspace-{id}")))
            .ghost()
            .small()
            .icon(Icon::new(Glyph::FolderSimplePlus))
            .tooltip(format!("New workspace on {}", machine.name))
            .on_click(move |_, window, cx| dialogs::new_workspace(&machine, window, cx))
            .into_any_element()
    }

    pub(super) fn machine(
        &mut self,
        machine: &Machine,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = store(cx);
        let connected = state.machines.read(cx).state(&machine.id) == State::Connected;
        let cache = state.workspaces.read(cx).cache(&machine.id);
        let disconnected = matches!(
            state.machines.read(cx).state(&machine.id),
            State::Disconnected { .. }
        );
        let failed = connected && cache.and_then(|cache| cache.error()).is_some();
        let down = disconnected || failed;
        let loading = cache.and_then(|cache| cache.list()).is_none() && !down;
        let error = if disconnected {
            match state.machines.read(cx).state(&machine.id) {
                State::Disconnected { error, .. } => Some(error),
                _ => None,
            }
        } else {
            cache.and_then(|cache| cache.error()).map(str::to_owned)
        };
        let workspaces = state.workspaces.read(cx).list(&machine.id).to_vec();
        let muted = cx.theme().muted_foreground;
        let icon = if machine.local() {
            Glyph::Desktop
        } else {
            Glyph::HardDrives
        };
        let menu_machine = machine.clone();

        let header = h_flex()
            .id(SharedString::from(format!("machine-{}", machine.id)))
            .group("machine")
            .h_10()
            .gap_2()
            .px_2()
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_2()
                    .type_title()
                    .text_color(muted)
                    .child(Icon::new(icon))
                    .child(div().truncate().child(machine.name.clone())),
            )
            .child(Self::action(machine, down, loading, error, cx));
        let header =
            if down || loading {
                header.into_any_element()
            } else {
                header
                    .context_menu(move |menu, _, _| {
                        let machine = menu_machine.clone();
                        menu.item(PopupMenuItem::new("New workspace").on_click(
                            move |_, window, cx| dialogs::new_workspace(&machine, window, cx),
                        ))
                    })
                    .into_any_element()
            };

        let items: Vec<AnyElement> = workspaces
            .iter()
            .map(|workspace| self.workspace(machine, workspace, down, window, cx))
            .collect();

        v_flex()
            .child(header)
            .child(v_flex().gap_1().children(items))
            .into_any_element()
    }
}
