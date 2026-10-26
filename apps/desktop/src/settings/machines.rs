use client::State;
use gpui_kit::component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_kit::component::{ActiveTheme, Icon, Sizable, WindowExt, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, px,
};
use protocol::Method;
use protocol::machines::{
    ConnectMachine, DisconnectMachine, MachineId, RemoveMachine, RenameMachine, RenameMachineParams,
};

use super::panel::Panel;
use super::{row, section as heading};
use crate::assets::Glyph;
use crate::notice;
use crate::runtime;
use crate::state::machines::{Machine, Machines};
use crate::state::store;
use crate::theme::{self, Type};

fn change<M>(machine: &Machine, failure: &'static str, cx: &mut App)
where
    M: Method<Params = MachineId, Result = ()>,
{
    let own = store(cx).machines.read(cx).own().clone();
    let request = runtime::call::<M>(&own, machine.key(), cx);
    let name = machine.name.clone();
    cx.spawn(async move |cx| {
        let result = request.await;
        cx.update(|cx| {
            if let Err(error) = result {
                notice::error(format!("{failure} {name}"), &error, cx);
            }
            let machines = store(cx).machines.clone();
            machines.update(cx, Machines::load);
        });
    })
    .detach();
}

fn remove(machine: &Machine, window: &mut Window, cx: &mut App) {
    let place = if machine.local() {
        "on this computer"
    } else {
        "on the server"
    };
    let title = format!("Remove {}?", machine.name);
    let description = format!(
        "Its workspaces, terminals and agents stay {place}. Add it again to pick up where you left off."
    );
    let machine = machine.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let machine = machine.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text("Remove")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                change::<RemoveMachine>(&machine, "Could not remove", cx);
                true
            })
    });
}

pub struct Rename {
    machine: Machine,
    name: Entity<InputState>,
    renaming: bool,
}

impl Rename {
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        self.renaming = true;
        cx.notify();
        let own = store(cx).machines.read(cx).own().clone();
        let params = RenameMachineParams {
            machine: self.machine.key(),
            name,
        };
        let request = runtime::call::<RenameMachine>(&own, params, cx);
        let label = self.machine.name.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.renaming = false;
                match result {
                    Ok(()) => {
                        window.close_dialog(cx);
                        let machines = store(cx).machines.clone();
                        machines.update(cx, Machines::load);
                    }
                    Err(error) => notice::error(format!("Could not rename {label}"), &error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for Rename {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(div().type_body().child("Name"))
                    .child(Input::new(&self.name)),
            )
            .child(
                h_flex().justify_end().child(
                    Button::new("rename")
                        .primary()
                        .small()
                        .loading(self.renaming)
                        .label(if self.renaming {
                            "Renaming…"
                        } else {
                            "Rename"
                        })
                        .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                ),
            )
    }
}

fn rename(machine: &Machine, window: &mut Window, cx: &mut App) {
    let placeholder = machine
        .target
        .clone()
        .unwrap_or_else(|| "This computer".into());
    let current = machine.name.clone();
    let form = cx.new(|cx| Rename {
        machine: machine.clone(),
        name: cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(current)
        }),
        renaming: false,
    });
    let title = format!("Rename {}", machine.name);
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(420.)).child(form.clone())
    });
}

fn menu(menu: PopupMenu, machine: &Machine, connected: bool, only: bool) -> PopupMenu {
    let renamed = machine.clone();
    let mut menu = menu.item(
        PopupMenuItem::new("Rename").on_click(move |_, window, cx| rename(&renamed, window, cx)),
    );
    if !only {
        let toggled = machine.clone();
        let removed = machine.clone();
        menu = menu
            .item(
                PopupMenuItem::new(if connected { "Disconnect" } else { "Connect" }).on_click(
                    move |_, _, cx| {
                        if connected {
                            change::<DisconnectMachine>(&toggled, "Could not disconnect", cx);
                        } else {
                            change::<ConnectMachine>(&toggled, "Could not connect", cx);
                        }
                    },
                ),
            )
            .separator()
            .item(
                PopupMenuItem::new("Remove")
                    .on_click(move |_, window, cx| remove(&removed, window, cx)),
            );
    }
    menu
}

fn dot(machine: &Machine, cx: &App) -> AnyElement {
    let tokens = theme::tokens(cx);
    let muted = cx.theme().muted_foreground;
    let color = if machine.connected {
        match store(cx).machines.read(cx).state(&machine.id) {
            State::Connected => tokens.success,
            State::Disconnected { .. } => tokens.destructive,
            State::Connecting | State::Idle => muted,
        }
    } else {
        muted
    };
    div()
        .size_2()
        .flex_shrink_0()
        .rounded_full()
        .bg(color)
        .into_any_element()
}

pub fn section(cx: &mut Context<Panel>) -> AnyElement {
    let machines: Vec<Machine> = store(cx).machines.read(cx).all().to_vec();
    let connected = machines.iter().filter(|machine| machine.connected).count();
    let add = Button::new("add-machine")
        .ghost()
        .small()
        .icon(Icon::new(Glyph::Plus))
        .tooltip("Add a machine")
        .on_click(|_, window, cx| {
            window.close_dialog(cx);
            if let Some(app) = crate::app::current(cx) {
                app.update(cx, |app, cx| app.add_machine(window, cx));
            }
        });

    let mut list = heading("Machines", None, Some(add.into_any_element()));
    for machine in machines {
        let only = machine.connected && connected == 1;
        let label = match machine
            .target
            .as_ref()
            .filter(|target| **target != machine.name)
        {
            Some(target) => format!("{} on {target}", machine.name),
            None => machine.name.to_string(),
        };
        let target = machine.clone();
        list = list.child(
            h_flex().gap_2().child(dot(&machine, cx)).child(
                div().flex_1().child(row(
                    label,
                    Button::new(SharedString::from(format!("machine-{}", machine.id)))
                        .ghost()
                        .small()
                        .icon(Icon::new(Glyph::DotsThree))
                        .tooltip(format!("Actions for {}", machine.name))
                        .dropdown_menu(move |popup, _, _| {
                            menu(popup, &target, target.connected, only)
                        }),
                )),
            ),
        );
    }
    list.into_any_element()
}
