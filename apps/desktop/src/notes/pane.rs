mod rows;

use gpui_kit::component::WindowExt;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::{App, AppContext, ClipboardItem, Entity, Window};

use super::name::{Prompt, ask};
use super::tree::Order;
use super::view::NotesTile;
use crate::persist;
use crate::state::notes::Kind;
use crate::state::store;

fn order(cx: &App) -> Order {
    persist::saved(cx)
        .sort
        .as_deref()
        .and_then(|order| serde_json::from_value(serde_json::Value::String(order.to_owned())).ok())
        .unwrap_or_default()
}

fn set_order(order: Order, cx: &mut App) {
    if let Ok(serde_json::Value::String(text)) = serde_json::to_value(order) {
        persist::update(cx, |saved| saved.sort = Some(text));
    }
}

pub fn create(
    tile: &Entity<NotesTile>,
    kind: Kind,
    parent: String,
    window: &mut Window,
    cx: &mut App,
) {
    let (heading, placeholder) = match kind {
        Kind::Note => ("New note", "Login flow"),
        Kind::Folder => ("New folder", "specs"),
    };
    let tile = tile.downgrade();
    let prompt = Prompt {
        title: heading.into(),
        action: "Create",
        busy: "Creating…",
        placeholder,
        initial: String::new(),
    };
    ask(prompt, window, cx, move |name, window, cx| {
        let Some(view) = tile.upgrade() else {
            return gpui_kit::Task::ready(false);
        };
        let machine = view.read(cx).machine.clone();
        let task = store(cx).notes.clone().update(cx, |notes, cx| {
            notes.create(&machine, kind, parent.clone(), name, cx)
        });
        let tile = tile.clone();
        window.spawn(cx, async move |cx| {
            let Some(path) = task.await else {
                return false;
            };
            if kind == Kind::Note {
                cx.update(|window, cx| {
                    tile.update(cx, |tile, cx| tile.open(&path, window, cx))
                        .ok();
                })
                .ok();
            }
            true
        })
    });
}

fn rename(
    machine: String,
    path: String,
    name: String,
    folder: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let placeholder = if folder { "Folder name" } else { "Note name" };
    let prompt = Prompt {
        title: format!("Rename {name}").into(),
        action: "Rename",
        busy: "Renaming…",
        placeholder,
        initial: name,
    };
    ask(prompt, window, cx, move |next, _, cx| {
        let task = store(cx).notes.clone().update(cx, |notes, cx| {
            notes.rename(&machine, path.clone(), next, cx)
        });
        cx.background_spawn(async move { task.await.is_some() })
    });
}

fn delete(
    machine: String,
    path: String,
    name: &str,
    folder: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let description = if folder {
        "It moves with everything in it to the .trash folder in your notes folder."
    } else {
        "It moves to the .trash folder in your notes folder."
    };
    let title = format!("Delete {name}?");
    window.open_alert_dialog(cx, move |alert, _, _| {
        let (machine, path) = (machine.clone(), path.clone());
        alert
            .title(title.clone())
            .description(description)
            .show_cancel(true)
            .ok_text("Delete")
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                let (machine, path) = (machine.clone(), path.clone());
                store(cx)
                    .notes
                    .clone()
                    .update(cx, |notes, cx| notes.delete(&machine, path, cx));
                true
            })
    });
}

fn menu(
    menu: PopupMenu,
    tile: &Entity<NotesTile>,
    machine: &str,
    path: &str,
    name: &str,
    folder: bool,
) -> PopupMenu {
    let mut menu = menu;
    if folder {
        let (note_tile, folder_tile) = (tile.clone(), tile.clone());
        let (note_parent, folder_parent) = (path.to_owned(), path.to_owned());
        menu = menu
            .item(
                PopupMenuItem::new("New note").on_click(move |_, window, cx| {
                    create(&note_tile, Kind::Note, note_parent.clone(), window, cx);
                }),
            )
            .item(
                PopupMenuItem::new("New folder").on_click(move |_, window, cx| {
                    create(
                        &folder_tile,
                        Kind::Folder,
                        folder_parent.clone(),
                        window,
                        cx,
                    );
                }),
            )
            .separator();
    }
    let copied = path.to_owned();
    let (rename_machine, rename_path, rename_name) =
        (machine.to_owned(), path.to_owned(), name.to_owned());
    let (delete_machine, delete_path, delete_name) =
        (machine.to_owned(), path.to_owned(), name.to_owned());
    menu.item(PopupMenuItem::new("Copy path").on_click(move |_, _, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
    }))
    .item(PopupMenuItem::new("Rename").on_click(move |_, window, cx| {
        rename(
            rename_machine.clone(),
            rename_path.clone(),
            rename_name.clone(),
            folder,
            window,
            cx,
        );
    }))
    .separator()
    .item(PopupMenuItem::new("Delete").on_click(move |_, window, cx| {
        delete(
            delete_machine.clone(),
            delete_path.clone(),
            &delete_name,
            folder,
            window,
            cx,
        );
    }))
}
