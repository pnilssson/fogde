use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{ContextMenuExt, DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use protocol::workspaces::{Checkout, Workspace};

use super::Sidebar;
use super::new_worktree;
use super::row::row;
use crate::assets::Glyph;
use crate::paths;
use crate::state::machines::Machine;
use crate::workspaces::desk::Desk;
use crate::workspaces::dialogs;
use crate::workspaces::selection::{main_checkout, worktrees};
use crate::workspaces::status::attention;
use crate::workspaces::status_dot;
use crate::workspaces::tiles::tile::View;

fn menu(
    menu: PopupMenu,
    desk: &Entity<Desk>,
    machine: &Machine,
    workspace: &Workspace,
) -> PopupMenu {
    let view = View::workspace(machine.id.to_string(), workspace.id.clone());
    let mut menu = menu;
    if !workspace.missing {
        let (folder, start) = (workspace.folder.clone(), desk.clone());
        menu = menu.item(
            PopupMenuItem::new("New tile").on_click(move |_, window, cx| {
                let (view, folder) = (view.clone(), folder.clone());
                start.update(cx, |desk, cx| desk.open_start(view, &folder, window, cx));
            }),
        );
        if main_checkout(workspace).is_some() {
            let (machine, workspace) = (machine.clone(), workspace.clone());
            menu = menu.item(
                PopupMenuItem::new("New worktree").on_click(move |_, window, cx| {
                    new_worktree::open(&machine, &workspace, window, cx);
                }),
            );
        }
        menu = menu.separator();
    }
    let (machine, workspace) = (machine.id.to_string(), workspace.clone());
    menu.item(PopupMenuItem::new("Delete").on_click(move |_, window, cx| {
        dialogs::delete_workspace(&machine, &workspace, window, cx);
    }))
}

fn worktree_menu(
    menu: PopupMenu,
    desk: Entity<Desk>,
    machine: &str,
    workspace: &Workspace,
    checkout: &Checkout,
) -> PopupMenu {
    let view = View {
        checkout: Some(checkout.path.clone()),
        ..View::workspace(machine, workspace.id.clone())
    };
    let (path, start) = (checkout.path.clone(), desk.clone());
    let (remove_view, remove_workspace) = (view.clone(), workspace.clone());
    menu.item(
        PopupMenuItem::new("New tile").on_click(move |_, window, cx| {
            let (view, path) = (view.clone(), path.clone());
            start.update(cx, |desk, cx| desk.open_start(view, &path, window, cx));
        }),
    )
    .separator()
    .item(PopupMenuItem::new("Remove").on_click(move |_, window, cx| {
        remove(&remove_view, &remove_workspace, desk.clone(), window, cx);
    }))
}

pub fn remove(
    view: &View,
    workspace: &Workspace,
    desk: Entity<Desk>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(path) = view.checkout.clone() else {
        return;
    };
    let workspaces = crate::state::store(cx).workspaces.clone();
    let task = workspaces.update(cx, |workspaces, cx| {
        workspaces.remove_worktree(&view.machine, workspace, &path, cx)
    });
    let gone = view.clone();
    window
        .spawn(cx, async move |cx| {
            if task.await {
                cx.update(|window, cx| {
                    desk.update(cx, |desk, cx| desk.worktree_removed(&gone, window, cx));
                })
                .ok();
            }
        })
        .detach();
}

impl Sidebar {
    pub(super) fn workspace(
        &mut self,
        machine: &Machine,
        workspace: &Workspace,
        down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let desk = self.desk.clone();
        let home = View::workspace(machine.id.to_string(), workspace.id.clone());
        let selected = desk.read(cx).view().cloned();
        let current = selected.as_ref() == Some(&home);
        let here = selected.as_ref().is_some_and(|view| view.home() == home);
        let open = desk.read(cx).board().is_open(&home);
        let muted = cx.theme().muted_foreground;
        let status = (!open && !down)
            .then(|| status_dot(attention(&workspace.terminals), cx))
            .flatten();

        let toggle = {
            let (desk, home) = (desk.clone(), home.clone());
            move |_: &gpui_kit::ClickEvent, window: &mut Window, cx: &mut App| {
                desk.update(cx, |desk, cx| {
                    if current {
                        let open = desk.board().is_open(&home);
                        desk.edit(window, cx, |board| board.set_open(&home, !open));
                    } else {
                        desk.select(home.clone(), window, cx);
                    }
                });
            }
        };

        let label = div()
            .when(workspace.missing, |label| label.text_color(muted))
            .child(workspace.name.clone());
        let icon = if open {
            Glyph::CaretDown
        } else {
            Glyph::FolderSimple
        };
        let (menu_desk, menu_machine, menu_workspace) =
            (desk.clone(), machine.clone(), workspace.clone());
        let (more_desk, more_machine, more_workspace) =
            (desk.clone(), machine.clone(), workspace.clone());

        let header = row(
            SharedString::from(format!("workspace-{}-{}", machine.id, workspace.id)),
            Icon::new(icon).small().into_any_element(),
            label,
            here,
            false,
            cx,
        )
        .group("workspace")
        .children(status)
        .when(!down, |header| {
            header.child(
                div()
                    .invisible()
                    .group_hover("workspace", Styled::visible)
                    .child(
                        Button::new(SharedString::from(format!("more-{}", workspace.id)))
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(Glyph::DotsThree))
                            .tooltip(format!("Actions for {}", workspace.name))
                            .dropdown_menu(move |popup, _, _| {
                                menu(popup, &more_desk, &more_machine, &more_workspace)
                            }),
                    ),
            )
        })
        .on_click(toggle);
        let header = if down {
            header.into_any_element()
        } else {
            header
                .context_menu(move |popup, _, _| {
                    menu(popup, &menu_desk, &menu_machine, &menu_workspace)
                })
                .into_any_element()
        };

        let mut children = v_flex().pl_4().gap_px();
        if open {
            children = children.children(self.rows(machine, workspace, &home, window, cx));
            for checkout in worktrees(workspace) {
                children = children
                    .children(self.worktree(machine, workspace, checkout, down, window, cx));
            }
        }

        v_flex().child(header).child(children).into_any_element()
    }

    fn worktree(
        &mut self,
        machine: &Machine,
        workspace: &Workspace,
        checkout: &Checkout,
        down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let selected = self.desk.read(cx).view().cloned();
        let view = View {
            checkout: Some(checkout.path.clone()),
            ..View::workspace(machine.id.to_string(), workspace.id.clone())
        };
        let active = selected.as_ref() == Some(&view);
        let name = checkout
            .branch
            .clone()
            .unwrap_or_else(|| paths::name(&checkout.path).to_owned());
        let muted = cx.theme().muted_foreground;
        let desk = self.desk.clone();
        let (menu_desk, more_desk) = (desk.clone(), desk.clone());
        let (menu_workspace, more_workspace) = (workspace.clone(), workspace.clone());
        let (menu_checkout, more_checkout) = (checkout.clone(), checkout.clone());
        let (menu_machine, more_machine) = (machine.id.to_string(), machine.id.to_string());
        let select = view.clone();

        let header = row(
            SharedString::from(format!("worktree-{}", checkout.path)),
            Icon::new(Glyph::GitBranch)
                .small()
                .text_color(muted)
                .into_any_element(),
            div().text_color(muted).child(name.clone()),
            active,
            true,
            cx,
        )
        .group("worktree")
        .on_click(move |_, window, cx| {
            let view = select.clone();
            desk.update(cx, |desk, cx| desk.select(view, window, cx));
        });
        let header = if down {
            header.into_any_element()
        } else {
            header
                .child(
                    div()
                        .invisible()
                        .group_hover("worktree", Styled::visible)
                        .child(
                            Button::new(SharedString::from(format!("more-{}", checkout.path)))
                                .ghost()
                                .xsmall()
                                .icon(Icon::new(Glyph::DotsThree))
                                .tooltip(format!("Actions for {name}"))
                                .dropdown_menu(move |popup, _, _| {
                                    worktree_menu(
                                        popup,
                                        more_desk.clone(),
                                        &more_machine,
                                        &more_workspace,
                                        &more_checkout,
                                    )
                                }),
                        ),
                )
                .context_menu(move |popup, _, _| {
                    worktree_menu(
                        popup,
                        menu_desk.clone(),
                        &menu_machine,
                        &menu_workspace,
                        &menu_checkout,
                    )
                })
                .into_any_element()
        };

        let mut rows = vec![h_flex().child(header).into_any_element()];
        rows.extend(self.rows(machine, workspace, &view, window, cx));
        rows
    }
}
