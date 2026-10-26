mod actions;
mod changes;
mod dnd;
mod render;
mod save;
pub mod split;
mod viewing;

use std::collections::{BTreeMap, HashMap};

use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    SharedString, Subscription, Task, WeakEntity, Window,
};
use protocol::workspaces::Workspace;

use super::selection::{self, Group, in_view};
use super::start::StartPage;
use super::tiles::Board;
use super::tiles::tile::{Selection, Tile, View};
use super::tiles::tree::Layout;
use crate::notes::view::NotesTile;
use crate::persist;
use crate::state::store;
use crate::terminals::TerminalView;

pub use changes::current;
pub use dnd::Hover;

pub enum Content {
    Terminal(Entity<TerminalView>),
    Start(Entity<StartPage>),
    Notes(Entity<NotesTile>),
}

pub enum DeskEvent {
    Changed,
}

pub struct Desk {
    board: Board,
    contents: HashMap<String, Content>,
    bounds: Option<Bounds<Pixels>>,
    hover: Option<Hover>,
    focus: FocusHandle,
    weak: WeakEntity<Self>,
    empty: Option<Entity<StartPage>>,
    focused_window: bool,
    viewing: viewing::Viewing,
    _subscriptions: Vec<Subscription>,
    _saving: Option<Task<()>>,
}

impl EventEmitter<DeskEvent> for Desk {}

pub fn later(
    desk: &WeakEntity<Desk>,
    window: &mut Window,
    cx: &mut App,
    change: impl FnOnce(&mut Desk, &mut Window, &mut Context<Desk>) + 'static,
) {
    let desk = desk.clone();
    window.defer(cx, move |window, cx| {
        desk.update(cx, |desk, cx| change(desk, window, cx)).ok();
    });
}

fn saved_layouts(cx: &App) -> BTreeMap<String, Layout> {
    persist::saved(cx)
        .layouts
        .iter()
        .filter_map(|(key, value)| {
            serde_json::from_value(value.clone())
                .ok()
                .map(|layout| (key.clone(), layout))
        })
        .collect()
}

impl Desk {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut board = Board::new(saved_layouts(cx), persist::saved(cx).closed.clone());
        if let Some((machine, id)) = persist::saved(cx).workspace.clone() {
            let checkout = persist::saved(cx).checkout.clone();
            board.select(View {
                machine,
                id,
                checkout,
            });
        }

        let state = store(cx);
        let workspaces = state.workspaces.clone();
        let machines = state.machines.clone();
        let notes = state.notes.clone();
        let subscriptions = vec![
            cx.subscribe_in(
                &notes,
                window,
                |this, _, change: &crate::state::notes::NotesChange, window, cx| {
                    if let crate::state::notes::NotesChange::Renamed { machine, from, to } = change
                    {
                        this.follow_rename(machine, from, to, window, cx);
                    }
                },
            ),
            cx.observe_in(&workspaces, window, |this, _, window, cx| {
                this.refresh(window, cx);
            }),
            cx.observe_in(&machines, window, |this, _, window, cx| {
                this.refresh(window, cx);
            }),
            cx.observe_window_activation(window, |this, window, cx| {
                this.focused_window = window.is_window_active();
                this.view_changed(cx);
            }),
        ];

        changes::register(cx.entity().downgrade(), cx);
        let mut desk = Self {
            board,
            contents: HashMap::new(),
            bounds: None,
            hover: None,
            focus: cx.focus_handle(),
            weak: cx.entity().downgrade(),
            empty: None,
            focused_window: window.is_window_active(),
            viewing: viewing::Viewing::new(cx),
            _subscriptions: subscriptions,
            _saving: None,
        };
        desk.refresh(window, cx);
        desk
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn view(&self) -> Option<&View> {
        self.board.view()
    }

    fn groups(cx: &App) -> Vec<(SharedString, Option<&[Workspace]>)> {
        let state = store(cx);
        let workspaces = state.workspaces.read(cx);
        state
            .machines
            .read(cx)
            .connected()
            .map(|machine| {
                let list = workspaces.cache(&machine.id).and_then(|cache| cache.list());
                (machine.id.clone(), list)
            })
            .collect()
    }

    pub fn workspace<'a>(&self, cx: &'a App) -> Option<&'a Workspace> {
        let view = self.board.view()?;
        store(cx)
            .workspaces
            .read(cx)
            .workspace(&view.machine, &view.id)
    }

    fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let groups = Self::groups(cx);
        let machines: Vec<&str> = groups.iter().map(|(id, _)| id.as_ref()).collect();
        let all_loaded = store(cx).machines.read(cx).loaded();
        if all_loaded && !machines.is_empty() {
            self.board.keep_machines(&machines);
        }

        let groups: Vec<Group> = groups
            .iter()
            .map(|(machine, workspaces)| Group {
                machine,
                workspaces: *workspaces,
            })
            .collect();
        if let Some(view) = selection::resolve(&groups, self.board.view())
            && self.board.view() != Some(&view)
        {
            self.board.select(view);
        }

        if let Some(view) = self.board.view().cloned()
            && let Some(workspace) = self.workspace(cx).cloned()
        {
            let terminals: Vec<Selection> = workspace
                .terminals
                .iter()
                .filter(|terminal| in_view(&view, terminal))
                .map(|terminal| Selection::new(view.machine.clone(), terminal.id.clone()))
                .collect();
            let workspaces = store(cx).workspaces.read(cx);
            self.board.sync(&terminals, |selection| {
                workspaces.placing(&selection.machine, &selection.id)
            });
        }

        self.ensure_contents(window, cx);
        self.changed(cx);
    }

    fn ensure_contents(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tiles: Vec<Tile> = self.board.layout().tiles().into_iter().cloned().collect();
        self.contents
            .retain(|id, _| tiles.iter().any(|tile| tile.id() == *id));

        for tile in tiles {
            let id = tile.id();
            if let Some(content) = self.contents.get(&id) {
                if let (Content::Start(start), Tile::Start { folder }) = (content, &tile) {
                    let folder = folder.clone();
                    start.update(cx, |start, cx| start.set_folder(folder, cx));
                }
                if let (Content::Notes(notes), Tile::Notes { note, .. }) = (content, &tile) {
                    let note = note.clone();
                    notes.update(cx, |notes, cx| notes.set_note(note, window, cx));
                }
                continue;
            }

            let content = match &tile {
                Tile::Terminal { machine, id } => {
                    let Some(client) = store(cx)
                        .machines
                        .read(cx)
                        .get(machine)
                        .map(|machine| machine.client.clone())
                    else {
                        continue;
                    };
                    let machine = SharedString::from(machine.clone());
                    let id = id.clone();
                    Content::Terminal(cx.new(|cx| TerminalView::new(machine, id, client, cx)))
                }
                Tile::Start { folder } => {
                    let desk = cx.entity().downgrade();
                    let folder = folder.clone();
                    Content::Start(cx.new(|cx| StartPage::new(desk, folder, window, cx)))
                }
                Tile::Notes { machine, note, key } => {
                    let desk = cx.entity().downgrade();
                    let (machine, note, key) = (machine.clone(), note.clone(), key.clone());
                    Content::Notes(
                        cx.new(|cx| NotesTile::new(desk, key, machine, note, window, cx)),
                    )
                }
            };
            self.contents.insert(id, content);
        }

        let active = self.board.active().map(str::to_owned);
        for (id, content) in &self.contents {
            if let Content::Terminal(terminal) = content {
                let is_active = active.as_deref() == Some(id.as_str());
                terminal.update(cx, |terminal, _| terminal.set_active(is_active));
            }
        }
    }
}

impl Focusable for Desk {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
