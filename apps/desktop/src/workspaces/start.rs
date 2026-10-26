mod render;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, Subscription, WeakEntity, Window,
};
use protocol::agents::Agent;
use protocol::terminals::CreateTerminalParams;
use protocol::workspaces::Workspace;
use protocol::worktrees::CreateWorktreeParams;

use super::desk::{self, Desk};
use super::selection::{folder_of, main_checkout};
use super::tiles::tile::{START, Selection, View};
use crate::state::machines::Machine;
use crate::state::store;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Launch(Option<Agent>);

pub struct StartPage {
    desk: WeakEntity<Desk>,
    folder: String,
    focus: FocusHandle,
    first: FocusHandle,
    name: Entity<InputState>,
    invalid: bool,
    opening: Option<Launch>,
    creating: bool,
    removing: bool,
    _subscriptions: Vec<Subscription>,
}

impl StartPage {
    pub fn new(
        desk: WeakEntity<Desk>,
        folder: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("fix-login"));
        let subscriptions = vec![
            cx.subscribe_in(&name, window, |this, _, event, window, cx| match event {
                InputEvent::PressEnter { .. } => this.create_worktree(window, cx),
                InputEvent::Change => {
                    this.invalid = false;
                    cx.notify();
                }
                _ => {}
            }),
        ];
        Self {
            desk,
            folder,
            focus: cx.focus_handle(),
            first: cx.focus_handle(),
            name,
            invalid: false,
            opening: None,
            creating: false,
            removing: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_folder(&mut self, folder: String, cx: &mut Context<Self>) {
        if self.folder != folder {
            self.folder = folder;
            cx.notify();
        }
    }

    fn context(&self, cx: &App) -> Option<(View, Machine, Workspace, String)> {
        let desk = self.desk.upgrade()?;
        let desk = desk.read(cx);
        let view = desk.view()?.clone();
        let machine = store(cx).machines.read(cx).get(&view.machine)?.clone();
        let workspace = desk.workspace(cx)?.clone();
        let folder = if self.folder.is_empty() {
            folder_of(&workspace, &view)
        } else {
            self.folder.clone()
        };
        Some((view, machine, workspace, folder))
    }

    fn open(&mut self, agent: Option<Agent>, window: &mut Window, cx: &mut Context<Self>) {
        let Some((view, _, workspace, folder)) = self.context(cx) else {
            return;
        };
        self.opening = Some(Launch(agent));
        cx.notify();

        let params = CreateTerminalParams {
            workspace: workspace.id,
            folder,
            agent,
        };
        let workspaces = store(cx).workspaces.clone();
        let task = workspaces.update(cx, |workspaces, cx| {
            workspaces.create_terminal(&view.machine, params, cx)
        });
        let desk = self.desk.clone();
        cx.spawn_in(window, async move |this, cx| {
            let terminal = task.await;
            this.update_in(cx, |this, window, cx| {
                this.opening = None;
                cx.notify();
                if let Some(terminal) = terminal {
                    let selection = Selection::new(view.machine.clone(), terminal.id);
                    desk::later(&desk, window, cx, move |desk, window, cx| {
                        desk.edit(window, cx, |board| board.opened(selection));
                    });
                }
            })
            .ok();
        })
        .detach();
    }

    fn notes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((view, ..)) = self.context(cx) else {
            return;
        };
        desk::later(&self.desk, window, cx, move |desk, window, cx| {
            desk.edit(window, cx, |board| {
                board.open_notes(&view.machine, None, crate::notes::key());
            });
        });
    }

    fn create_worktree(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.invalid = true;
            cx.notify();
            return;
        }
        let Some((view, _, workspace, _)) = self.context(cx) else {
            return;
        };
        let Some(repo) = main_checkout(&workspace).map(|main| main.path.clone()) else {
            return;
        };
        self.creating = true;
        cx.notify();

        let workspaces = store(cx).workspaces.clone();
        let task = workspaces.update(cx, |workspaces, cx| {
            workspaces.create_worktree(&view.machine, CreateWorktreeParams { repo, name }, cx)
        });
        let desk = self.desk.clone();
        cx.spawn_in(window, async move |this, cx| {
            let path = task.await;
            this.update_in(cx, |this, window, cx| {
                this.creating = false;
                cx.notify();
                let Some(path) = path else {
                    return;
                };
                this.name
                    .update(cx, |name, cx| name.set_value("", window, cx));
                let next = View {
                    checkout: Some(path),
                    ..view.home()
                };
                desk::later(&desk, window, cx, move |desk, window, cx| {
                    desk.close(START, None, window, cx);
                    desk.select(next, window, cx);
                });
            })
            .ok();
        })
        .detach();
    }

    fn remove_worktree(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((view, _, workspace, folder)) = self.context(cx) else {
            return;
        };
        self.removing = true;
        cx.notify();

        let workspaces = store(cx).workspaces.clone();
        let task = workspaces.update(cx, |workspaces, cx| {
            workspaces.remove_worktree(&view.machine, &workspace, &folder, cx)
        });
        let desk = self.desk.clone();
        cx.spawn_in(window, async move |this, cx| {
            let removed = task.await;
            this.update_in(cx, |this, window, cx| {
                this.removing = false;
                cx.notify();
                if removed {
                    let gone = View {
                        checkout: Some(folder),
                        ..view.home()
                    };
                    desk::later(&desk, window, cx, move |desk, window, cx| {
                        desk.worktree_removed(&gone, window, cx);
                    });
                }
            })
            .ok();
        })
        .detach();
    }
}

impl Focusable for StartPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.first.clone()
    }
}
