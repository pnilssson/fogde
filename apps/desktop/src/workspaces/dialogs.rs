use std::rc::Rc;

use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::{App, WeakEntity, Window};
use protocol::repos::GetReposFolder;
use protocol::terminals::Terminal;
use protocol::workspaces::Workspace;

use super::desk;
use super::status::{activity, busy};
use super::tiles::tile::View;
use crate::folders::{self, FolderBrowser};
use crate::runtime;
use crate::state::machines::Machine;
use crate::state::store;

fn created(machine: String, workspace: Option<Workspace>, window: &mut Window, cx: &mut App) {
    let Some(workspace) = workspace else {
        return;
    };
    if let Some(desk) = desk::current(cx) {
        desk.update(cx, |desk, cx| {
            desk.select(View::workspace(machine, workspace.id), window, cx);
        });
    }
}

pub fn new_workspace(machine: &Machine, window: &mut Window, cx: &mut App) {
    let id = machine.id.to_string();
    let local = machine.local();
    let client = machine.client.clone();
    let repos = runtime::call::<GetReposFolder>(&client, (), cx);

    window
        .spawn(cx, async move |cx| {
            let start = repos.await.unwrap_or_default();
            if local {
                let mut dialog = rfd::AsyncFileDialog::new();
                if !start.is_empty() {
                    dialog = dialog.set_directory(&start);
                }
                let Some(picked) = dialog.pick_folder().await else {
                    return;
                };
                let folder = picked.path().display().to_string();
                cx.update(|window, cx| {
                    let task = store(cx).workspaces.clone().update(cx, |workspaces, cx| {
                        workspaces.create_workspace(&id, folder, cx)
                    });
                    window
                        .spawn(cx, async move |cx| {
                            let workspace = task.await;
                            cx.update(|window, cx| created(id, workspace, window, cx))
                                .ok();
                        })
                        .detach();
                })
                .ok();
                return;
            }

            cx.update(|window, cx| {
                let choose = Rc::new(
                    move |folder: String,
                          browser: WeakEntity<FolderBrowser>,
                          window: &mut Window,
                          cx: &mut App| {
                        let id = id.clone();
                        browser
                            .update(cx, |browser, cx| browser.set_busy(true, cx))
                            .ok();
                        let task = store(cx).workspaces.clone().update(cx, |workspaces, cx| {
                            workspaces.create_workspace(&id, folder, cx)
                        });
                        window
                            .spawn(cx, async move |cx| {
                                let workspace = task.await;
                                cx.update(|window, cx| {
                                    browser
                                        .update(cx, |browser, cx| browser.set_busy(false, cx))
                                        .ok();
                                    if workspace.is_some() {
                                        window.close_dialog(cx);
                                    }
                                    created(id, workspace, window, cx);
                                })
                                .ok();
                            })
                            .detach();
                    },
                );
                folders::open(
                    client,
                    start,
                    "New workspace",
                    "Create workspace",
                    choose,
                    window,
                    cx,
                );
            })
            .ok();
        })
        .detach();
}

fn outcome(count: usize) -> String {
    match count {
        0 => "This removes it from the sidebar.".to_owned(),
        1 => "This closes its terminal and removes it from the sidebar.".to_owned(),
        count => format!("This closes its {count} terminals and removes it from the sidebar."),
    }
}

pub fn delete_workspace(machine: &str, workspace: &Workspace, window: &mut Window, cx: &mut App) {
    let title = format!("Delete {}?", workspace.name);
    let description = format!(
        "{} Your files in {} aren't touched.",
        outcome(workspace.terminals.len()),
        workspace.folder
    );
    let (machine, id) = (machine.to_owned(), workspace.id.clone());

    window.open_alert_dialog(cx, move |alert, _, _| {
        let (machine, id) = (machine.clone(), id.clone());
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text("Delete")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, window, cx| {
                let task = store(cx).workspaces.clone().update(cx, |workspaces, cx| {
                    workspaces.delete_workspace(&machine, &id, cx)
                });
                let (machine, id) = (machine.clone(), id.clone());
                window
                    .spawn(cx, async move |cx| {
                        if task.await {
                            cx.update(|window, cx| {
                                if let Some(desk) = desk::current(cx) {
                                    desk.update(cx, |desk, cx| {
                                        desk.deleted(&machine, &id, window, cx);
                                    });
                                }
                            })
                            .ok();
                        }
                    })
                    .detach();
                true
            })
    });
}

pub fn close_terminal(machine: &str, terminal: &Terminal, window: &mut Window, cx: &mut App) {
    let close = {
        let (machine, terminal) = (machine.to_owned(), terminal.clone());
        move |cx: &mut App| {
            store(cx).workspaces.clone().update(cx, |workspaces, cx| {
                workspaces.close_terminal(&machine, &terminal, cx);
            });
        }
    };
    if !busy(terminal.status) {
        close(cx);
        return;
    }

    let who = terminal
        .agent
        .map_or("The terminal", |agent| agent.spec().name);
    let description = format!("{who} {}. Closing stops it.", activity(terminal.status));
    let title = format!("Close {}?", terminal.name);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let close = close.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text("Close")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                close(cx);
                true
            })
    });
}
