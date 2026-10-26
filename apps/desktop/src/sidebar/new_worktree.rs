use gpui_kit::component::WindowExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Sizable, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div, prelude::FluentBuilder, px,
};
use protocol::workspaces::Workspace;
use protocol::worktrees::CreateWorktreeParams;

use crate::state::machines::Machine;
use crate::state::store;
use crate::theme::Type;
use crate::workspaces::desk;
use crate::workspaces::selection::main_checkout;
use crate::workspaces::tiles::tile::View;

pub struct NewWorktree {
    machine: String,
    workspace: String,
    repo: String,
    name: Entity<InputState>,
    invalid: bool,
    creating: bool,
    _subscription: Subscription,
}

impl NewWorktree {
    fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.invalid = true;
            cx.notify();
            return;
        }
        self.creating = true;
        cx.notify();

        let params = CreateWorktreeParams {
            repo: self.repo.clone(),
            name,
        };
        let machine = self.machine.clone();
        let task = store(cx).workspaces.clone().update(cx, |workspaces, cx| {
            workspaces.create_worktree(&machine, params, cx)
        });
        let view = View::workspace(self.machine.clone(), self.workspace.clone());
        cx.spawn_in(window, async move |this, cx| {
            let path = task.await;
            this.update_in(cx, |this, window, cx| {
                this.creating = false;
                cx.notify();
                let Some(path) = path else {
                    return;
                };
                window.close_dialog(cx);
                if let Some(desk) = desk::current(cx) {
                    let next = View {
                        checkout: Some(path),
                        ..view
                    };
                    desk.update(cx, |desk, cx| desk.select(next, window, cx));
                }
            })
            .ok();
        })
        .detach();
    }
}

impl Render for NewWorktree {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(div().type_body().child("Name"))
            .child(
                Input::new(&self.name).suffix(
                    Button::new("create")
                        .ghost()
                        .xsmall()
                        .loading(self.creating)
                        .label(if self.creating {
                            "Creating…"
                        } else {
                            "Create"
                        })
                        .on_click(cx.listener(|this, _, window, cx| this.create(window, cx))),
                ),
            )
            .when(self.invalid, |form| {
                form.child(
                    div()
                        .type_caption()
                        .text_color(cx.theme().danger)
                        .child("Enter a name"),
                )
            })
    }
}

pub fn open(machine: &Machine, workspace: &Workspace, window: &mut Window, cx: &mut App) {
    let Some(repo) = main_checkout(workspace).map(|main| main.path.clone()) else {
        return;
    };
    let form = cx.new(|cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("fix-login"));
        let subscription = cx.subscribe_in(
            &name,
            window,
            |this: &mut NewWorktree, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.create(window, cx),
                InputEvent::Change => {
                    this.invalid = false;
                    cx.notify();
                }
                _ => {}
            },
        );
        NewWorktree {
            machine: machine.id.to_string(),
            workspace: workspace.id.clone(),
            repo,
            name,
            invalid: false,
            creating: false,
            _subscription: subscription,
        }
    });
    let focus = form.read(cx).name.clone();
    focus.update(cx, |name, cx| name.focus(window, cx));
    let title = format!("New worktree in {}", workspace.name);
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(420.)).child(form.clone())
    });
}
