use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable, Sizable, WindowExt};
use gpui_kit::{
    App, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px,
};
use protocol::notes::{GetNotesFolder, SetNotesFolder};

use super::{row, section};
use crate::folders;
use crate::notice;
use crate::runtime;
use crate::state::machines::Machine;
use crate::state::store;
use crate::theme::Type;

pub struct NotesFolders {
    folders: HashMap<SharedString, String>,
    saving: HashSet<SharedString>,
}

impl NotesFolders {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let machines: Vec<Machine> = store(cx).machines.read(cx).connected().cloned().collect();
        for machine in machines {
            let request = runtime::call::<GetNotesFolder>(&machine.client, (), cx);
            cx.spawn(async move |this, cx| {
                if let Ok(folder) = request.await {
                    this.update(cx, |this, cx| {
                        this.folders.insert(machine.id.clone(), folder);
                        cx.notify();
                    })
                    .ok();
                }
            })
            .detach();
        }
        Self {
            folders: HashMap::new(),
            saving: HashSet::new(),
        }
    }

    fn save(
        &mut self,
        machine: &Machine,
        path: String,
        close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.saving.insert(machine.id.clone());
        cx.notify();
        let request = runtime::call::<SetNotesFolder>(&machine.client, path.clone(), cx);
        let id = machine.id.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.saving.remove(&id);
                match result {
                    Ok(()) => {
                        this.folders.insert(id, path);
                        if close {
                            window.close_dialog(cx);
                        }
                    }
                    Err(error) => notice::error("Could not save the notes folder", &error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn change(&mut self, machine: Machine, window: &mut Window, cx: &mut Context<Self>) {
        let start = self.folders.get(&machine.id).cloned().unwrap_or_default();
        if machine.local() {
            cx.spawn_in(window, async move |this, cx| {
                let mut dialog = rfd::AsyncFileDialog::new();
                if !start.is_empty() {
                    dialog = dialog.set_directory(&start);
                }
                let Some(picked) = dialog.pick_folder().await else {
                    return;
                };
                let path = picked.path().display().to_string();
                this.update_in(cx, |this, window, cx| {
                    this.save(&machine, path, false, window, cx);
                })
                .ok();
            })
            .detach();
            return;
        }

        let me = cx.entity().downgrade();
        let target = machine.clone();
        let choose = Rc::new(move |path: String, _, window: &mut Window, cx: &mut App| {
            me.update(cx, |this, cx| this.save(&target, path, true, window, cx))
                .ok();
        });
        folders::open(
            machine.client,
            start,
            "Choose a notes folder",
            "Use this folder",
            choose,
            window,
            cx,
        );
    }
}

impl Render for NotesFolders {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let machines: Vec<Machine> = store(cx).machines.read(cx).connected().cloned().collect();
        let muted = cx.theme().muted_foreground;
        let mut notes = section(
            "Notes folder",
            Some("Use a folder you already have, like an Obsidian vault."),
            None,
        );
        for machine in machines {
            let folder = self.folders.get(&machine.id).cloned();
            let saving = self.saving.contains(&machine.id);
            let target = machine.clone();
            notes = notes.child(row(
                machine.name.clone(),
                gpui_kit::component::h_flex()
                    .gap_2()
                    .child(
                        div()
                            .max_w(px(256.))
                            .truncate()
                            .type_body()
                            .text_color(muted)
                            .child(folder.clone().unwrap_or_default()),
                    )
                    .child(
                        Button::new(SharedString::from(format!("notes-{}", machine.id)))
                            .ghost()
                            .small()
                            .loading(saving)
                            .disabled(folder.is_none() || saving)
                            .label(if saving { "Saving…" } else { "Change" })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.change(target.clone(), window, cx);
                            })),
                    ),
            ));
        }
        notes
    }
}
