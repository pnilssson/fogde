use std::collections::HashMap;

use gpui_kit::{App, Context, Entity, EventEmitter, SharedString, Subscription, Task};
use protocol::folders::CreateFolderParams;
use protocol::notes::{
    CreateNote, CreateNoteFolder, DeleteNote, ListNotes, NoteList, NotesEvent, RenameNote,
    RenameNoteParams,
};

use crate::notice;
use crate::persist;
use crate::runtime;
use crate::state::listen::drain;
use crate::state::machines::{Change, Machines};

pub enum NotesChange {
    Changed(SharedString),
    Renamed {
        machine: SharedString,
        from: String,
        to: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Note,
    Folder,
}

#[derive(Default)]
pub struct Entry {
    pub list: Option<NoteList>,
    pub error: Option<String>,
}

pub struct Notes {
    machines: Entity<Machines>,
    entries: HashMap<SharedString, Entry>,
    watches: HashMap<SharedString, Task<()>>,
    loads: HashMap<SharedString, Task<()>>,
    _machines: Subscription,
}

impl EventEmitter<NotesChange> for Notes {}

impl Notes {
    pub fn new(machines: Entity<Machines>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe(&machines, |this, _, change, cx| match change {
            Change::Connected(id) => {
                if this.entries.contains_key(id) {
                    this.reload(id, cx);
                }
            }
            Change::Listed => {
                let connected: Vec<SharedString> = this
                    .machines
                    .read(cx)
                    .connected()
                    .map(|machine| machine.id.clone())
                    .collect();
                this.entries.retain(|id, _| connected.contains(id));
                this.watches.retain(|id, _| connected.contains(id));
            }
        });
        Self {
            machines,
            entries: HashMap::new(),
            watches: HashMap::new(),
            loads: HashMap::new(),
            _machines: subscription,
        }
    }

    pub fn entry(&self, machine: &str) -> Option<&Entry> {
        self.entries.get(machine)
    }

    pub fn list(&self, machine: &str) -> Option<&NoteList> {
        self.entry(machine).and_then(|entry| entry.list.as_ref())
    }

    pub fn ensure(&mut self, machine: &str, cx: &mut Context<Self>) {
        if !self.entries.contains_key(machine) {
            self.reload(machine, cx);
        }
    }

    pub fn reload(&mut self, machine: &str, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        let id = SharedString::from(machine.to_owned());
        self.entries.entry(id.clone()).or_default();
        if !self.watches.contains_key(&id) {
            let changed = id.clone();
            let task = drain::<NotesEvent, _>(&client, cx, move |this, (), cx| {
                this.reload(&changed, cx);
                cx.emit(NotesChange::Changed(changed.clone()));
                true
            });
            self.watches.insert(id.clone(), task);
        }

        let request = runtime::call::<ListNotes>(&client, (), cx);
        let key = id.clone();
        self.loads.insert(
            id,
            cx.spawn(async move |this, cx| {
                let result = request.await;
                this.update(cx, |this, cx| {
                    let entry = this.entries.entry(key).or_default();
                    match result {
                        Ok(list) => {
                            entry.list = Some(list);
                            entry.error = None;
                        }
                        Err(error) => entry.error = Some(error.to_string()),
                    }
                    cx.notify();
                })
                .ok();
            }),
        );
    }

    pub fn create(
        &mut self,
        machine: &str,
        kind: Kind,
        parent: String,
        name: String,
        cx: &mut Context<Self>,
    ) -> Task<Option<String>> {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return Task::ready(None);
        };
        let params = CreateFolderParams { parent, name };
        let request = match kind {
            Kind::Note => runtime::call::<CreateNote>(&client, params, cx),
            Kind::Folder => runtime::call::<CreateNoteFolder>(&client, params, cx),
        };
        let machine = machine.to_owned();
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                this.reload(&machine, cx);
                result
                    .map_err(|error| notice::error("Could not create it", &error, cx))
                    .ok()
            })
            .ok()
            .flatten()
        })
    }

    pub fn rename(
        &mut self,
        machine: &str,
        path: String,
        name: String,
        cx: &mut Context<Self>,
    ) -> Task<Option<String>> {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return Task::ready(None);
        };
        let request = runtime::call::<RenameNote>(
            &client,
            RenameNoteParams {
                path: path.clone(),
                name,
            },
            cx,
        );
        let machine = SharedString::from(machine.to_owned());
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                this.reload(&machine, cx);
                match result {
                    Ok(to) => {
                        followed(&machine, &path, &to, cx);
                        cx.emit(NotesChange::Renamed {
                            machine: machine.clone(),
                            from: path,
                            to: to.clone(),
                        });
                        Some(to)
                    }
                    Err(error) => {
                        notice::error("Could not rename it", &error, cx);
                        None
                    }
                }
            })
            .ok()
            .flatten()
        })
    }

    pub fn delete(&mut self, machine: &str, path: String, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        let request = runtime::call::<DeleteNote>(&client, path, cx);
        let machine = machine.to_owned();
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    notice::error("Could not delete it", &error, cx);
                }
                this.reload(&machine, cx);
            })
            .ok();
        })
        .detach();
    }
}

fn followed(machine: &str, from: &str, to: &str, cx: &mut App) {
    let viewed = persist::saved(cx)
        .recent
        .get(machine)
        .cloned()
        .unwrap_or_default();
    let next = crate::notes::recent::follow(&viewed, from, to);
    if next != viewed {
        let machine = machine.to_owned();
        persist::update(cx, |saved| {
            saved.recent.insert(machine, next);
        });
    }
}

pub fn viewed(machine: &str, path: &str, cx: &mut App) {
    let list = persist::saved(cx)
        .recent
        .get(machine)
        .cloned()
        .unwrap_or_default();
    let next = crate::notes::recent::viewed(&list, path);
    if next != list {
        let machine = machine.to_owned();
        persist::update(cx, |saved| {
            saved.recent.insert(machine, next);
        });
    }
}
