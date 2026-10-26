mod render;

use std::collections::HashSet;
use std::rc::Rc;

use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, Subscription, Task, WeakEntity,
    Window,
};
use protocol::notes::ReadNote;

use super::editor::NoteEditor;
use super::links::{resolve_href, resolve_wikilink};
use super::renames::moved;
use super::tree::folders_above;
use super::wikilinks;
use crate::notice;
use crate::runtime;
use crate::state::notes::{NotesChange, viewed};
use crate::state::store;
use crate::workspaces::desk::{self, Desk};

pub struct NotesTile {
    pub(super) desk: WeakEntity<Desk>,
    pub(super) key: String,
    pub(super) machine: String,
    pub(super) note: Option<String>,
    pub(super) browsing: bool,
    pub(super) expanded: HashSet<String>,
    editor: Option<(String, Entity<NoteEditor>)>,
    reading: Option<Task<()>>,
    failed: bool,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl NotesTile {
    pub fn new(
        desk: WeakEntity<Desk>,
        key: String,
        machine: String,
        note: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let notes = store(cx).notes.clone();
        notes.update(cx, |notes, cx| notes.ensure(&machine, cx));
        let subscriptions = vec![
            cx.observe_in(&notes, window, |this, _, window, cx| this.sync(window, cx)),
            cx.subscribe_in(
                &notes,
                window,
                |this, _, change: &NotesChange, window, cx| match change {
                    NotesChange::Changed(machine) if *machine == this.machine => {
                        if let Some((_, editor)) = &this.editor {
                            editor.update(cx, |editor, cx| editor.reread(window, cx));
                        }
                    }
                    NotesChange::Renamed { machine, from, to } if *machine == this.machine => {
                        this.expanded = this
                            .expanded
                            .iter()
                            .map(|folder| moved(folder, from, to).unwrap_or_else(|| folder.clone()))
                            .collect();
                        if let Some(next) =
                            this.note.as_deref().and_then(|note| moved(note, from, to))
                        {
                            this.editor = None;
                            this.open(&next, window, cx);
                        }
                    }
                    _ => {}
                },
            ),
        ];
        let mut tile = Self {
            desk,
            key,
            machine,
            note: None,
            browsing: true,
            expanded: HashSet::new(),
            editor: None,
            reading: None,
            failed: false,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        tile.set_note(note, window, cx);
        tile
    }

    pub fn set_note(&mut self, note: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        if self.note == note {
            return;
        }
        self.note = note;
        self.editor = None;
        self.failed = false;
        self.sync(window, cx);
        cx.notify();
    }

    pub fn open(&mut self, path: &str, window: &mut Window, cx: &mut Context<Self>) {
        let (key, path) = (self.key.clone(), path.to_owned());
        desk::later(&self.desk, window, cx, move |desk, window, cx| {
            desk.edit(window, cx, |board| board.show_note(&key, &path));
        });
    }

    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.note.clone() else {
            return;
        };
        let Some(list) = store(cx).notes.read(cx).list(&self.machine) else {
            return;
        };
        if !list.notes.iter().any(|note| note.path == path) {
            return;
        }
        for folder in folders_above(&path, &list.root) {
            self.expanded.insert(folder);
        }
        if self.editor.as_ref().is_some_and(|(open, _)| *open == path) || self.reading.is_some() {
            return;
        }
        self.read(path, window, cx);
    }

    fn read(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(machine) = store(cx).machines.read(cx).get(&self.machine).cloned() else {
            return;
        };
        viewed(&self.machine, &path, cx);
        let request = runtime::call::<ReadNote>(&machine.client, path.clone(), cx);
        self.reading = Some(cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.reading = None;
                if this.note.as_deref() != Some(path.as_str()) {
                    return;
                }
                match result {
                    Ok(loaded) => {
                        let follow = Self::follow(cx);
                        let editor = cx.new(|cx| {
                            NoteEditor::new(machine, path.clone(), loaded, follow, window, cx)
                        });
                        this.editor = Some((path, editor));
                        this.failed = false;
                    }
                    Err(error) => {
                        this.failed = true;
                        notice::error("Could not read the note", &error, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn follow(cx: &mut Context<Self>) -> super::editor::Follow {
        let me = cx.entity().downgrade();
        Rc::new(move |offset, text, window, cx| {
            let Some(tile) = me.upgrade() else {
                return;
            };
            let target = link_at(text, offset);
            let (machine, note) = {
                let tile = tile.read(cx);
                (tile.machine.clone(), tile.note.clone())
            };
            let Some(list) = store(cx).notes.read(cx).list(&machine).cloned() else {
                return;
            };
            let from = list
                .notes
                .iter()
                .find(|known| Some(&known.path) == note.as_ref());
            let found = match &target {
                Some(Link::Wiki(target)) => {
                    resolve_wikilink(target, &list.notes, from).map(|note| note.path.clone())
                }
                Some(Link::Href(href)) => from
                    .and_then(|from| resolve_href(href, &list.notes, from))
                    .map(|note| note.path.clone()),
                None => None,
            };
            match (found, target) {
                (Some(path), _) => tile.update(cx, |tile, cx| tile.open(&path, window, cx)),
                (None, Some(Link::Href(href))) => {
                    let lower = href.to_ascii_lowercase();
                    if lower.starts_with("http://")
                        || lower.starts_with("https://")
                        || lower.starts_with("mailto:")
                    {
                        cx.open_url(&href);
                    }
                }
                _ => {}
            }
        })
    }
}

enum Link {
    Wiki(String),
    Href(String),
}

fn link_at(text: &str, offset: usize) -> Option<Link> {
    if let Some(link) = wikilinks::find(text)
        .into_iter()
        .find(|link| link.range.contains(&offset))
    {
        return Some(Link::Wiki(link.target));
    }
    let start = text[..offset.min(text.len())]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    let end = text[offset.min(text.len())..]
        .find('\n')
        .map_or(text.len(), |at| offset + at);
    let line = &text[start..end];
    let mut from = 0;
    while let Some(open) = line[from..].find("](").map(|at| from + at) {
        let close = line[open..].find(')').map(|at| open + at)?;
        let bracket = line[..open].rfind('[').unwrap_or(open);
        if (start + bracket..=start + close).contains(&offset) {
            return Some(Link::Href(line[open + 2..close].to_owned()));
        }
        from = close;
    }
    let word_start = line[..offset - start]
        .rfind(char::is_whitespace)
        .map_or(0, |at| at + 1);
    let word_end = line[offset - start..]
        .find(char::is_whitespace)
        .map_or(line.len(), |at| offset - start + at);
    let word = &line[word_start..word_end];
    (word.starts_with("http://") || word.starts_with("https://"))
        .then(|| Link::Href(word.to_owned()))
}

impl Focusable for NotesTile {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor
            .as_ref()
            .map_or_else(|| self.focus.clone(), |(_, editor)| editor.focus_handle(cx))
    }
}
