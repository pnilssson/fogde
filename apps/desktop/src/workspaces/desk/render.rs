use gpui_kit::component::{ActiveTheme, Icon, Sizable, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext, Bounds, Context, DragMoveEvent, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Render, SharedString, Styled, Window, canvas, div, px,
};

use super::{Content, Desk};
use crate::assets::Glyph;
use crate::state::store;
use crate::theme;
use crate::workspaces::dialogs;
use crate::workspaces::start::StartPage;
use crate::workspaces::terminal_icon;
use crate::workspaces::tile_bar::TileBar;
use crate::workspaces::tiles::tile::{Dragged, START, Tile};
use crate::workspaces::tiles::tree::Layout;

type Close = Box<dyn Fn(&gpui_kit::ClickEvent, &mut Window, &mut App)>;

impl Desk {
    fn close_button(&self, tile: &Tile, cx: &App) -> Option<Close> {
        let id = tile.id();
        if let Tile::Terminal {
            machine,
            id: terminal,
        } = tile
        {
            let state = store(cx);
            let connected = state.machines.read(cx).state(machine) == client::State::Connected;
            let workspaces = state.workspaces.read(cx);
            if !connected || workspaces.closing(machine, terminal) {
                return None;
            }
            let (_, found) = workspaces.terminal(machine, terminal)?;
            let (machine, found) = (machine.clone(), found.clone());
            Some(Box::new(move |_, window, cx| {
                dialogs::close_terminal(&machine, &found, window, cx);
            }))
        } else {
            let me = self.weak.clone();
            Some(Box::new(move |_, window, cx| {
                me.update(cx, |desk, cx| desk.close(&id, None, window, cx))
                    .ok();
            }))
        }
    }

    pub(super) fn tile(&self, tile: &Tile, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let id = tile.id();
        let active = self.board.active() == Some(id.as_str());
        let on_close = self.close_button(tile, cx);
        let shared = self.board.layout().len() > 1;

        let (bar, content): (TileBar, AnyElement) = match (tile, self.contents.get(&id)) {
            (
                Tile::Terminal {
                    machine,
                    id: terminal,
                },
                Some(Content::Terminal(view)),
            ) => {
                let workspaces = store(cx).workspaces.read(cx);
                let Some((workspace, found)) = workspaces.terminal(machine, terminal) else {
                    return div().into_any_element();
                };
                let home = found
                    .checkout
                    .clone()
                    .unwrap_or_else(|| workspace.folder.clone());
                let place = crate::workspaces::tiles::Board::place_label(&found.folder, &home)
                    .map(|label| (label, found.folder.clone()));
                let selection = tile.selection().map(Dragged::Terminal);
                let bar = TileBar::new(
                    SharedString::from(format!("bar-{id}")),
                    terminal_icon(found.agent, cx),
                    found.name.clone(),
                )
                .dragged(selection)
                .status(found.status)
                .place(place)
                .branch(found.branch.clone())
                .changes(found.changes.clone());
                (
                    bar,
                    view.clone()
                        .cached(gpui_kit::StyleRefinement::default().size_full())
                        .into_any_element(),
                )
            }
            (Tile::Start { .. }, Some(Content::Start(view))) => {
                let dragged = shared.then(|| Dragged::Tile {
                    id: START.into(),
                    view: None,
                });
                let bar = TileBar::new(
                    "bar-start",
                    Icon::new(Glyph::PlusCircle).small(),
                    "New tile",
                )
                .dragged(dragged);
                (bar, view.clone().into_any_element())
            }
            (Tile::Notes { note, .. }, Some(Content::Notes(view))) => {
                let name = crate::workspaces::tiles::tile::note_title(note.as_deref());
                let dragged = Some(Dragged::Tile {
                    id: id.clone(),
                    view: None,
                });
                let bar = TileBar::new(
                    SharedString::from(format!("bar-{id}")),
                    Icon::new(Glyph::Notebook).small(),
                    name,
                )
                .dragged(dragged);
                (bar, view.clone().into_any_element())
            }
            _ => return div().into_any_element(),
        };

        let activate = id.clone();
        v_flex()
            .id(SharedString::from(format!("tile-{id}")))
            .size_full()
            .min_w_0()
            .min_h_0()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |desk, _, window, cx| {
                    if desk.board.active() != Some(activate.as_str()) {
                        desk.activate(&activate, window, cx);
                    }
                }),
            )
            .child(bar.active(active).on_close(on_close))
            .child(div().flex_1().min_h_0().child(content))
            .into_any_element()
    }

    fn overlay(&self, bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement> {
        let hover = self.hover.as_ref().filter(|hover| hover.allowed)?;
        let shade = hover.shade();
        let tint = theme::tokens(cx).selected;
        Some(
            div()
                .absolute()
                .left(px(shade.x) - bounds.origin.x)
                .top(px(shade.y) - bounds.origin.y)
                .w(px(shade.width))
                .h(px(shade.height))
                .rounded(cx.theme().radius_lg)
                .bg(tint)
                .border_1()
                .border_color(theme::tokens(cx).border)
                .into_any_element(),
        )
    }

    fn trash(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let dragging = self.hover.as_ref().map(|hover| hover.dragged.clone())?;
        if matches!(dragging, Dragged::Note { .. }) {
            return None;
        }
        let tokens = theme::tokens(cx);
        let destructive = tokens.destructive;

        Some(
            div()
                .absolute()
                .bottom_4()
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    div()
                        .id("trash")
                        .size_12()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(cx.theme().popover)
                        .shadow(tokens.shadows[4].clone())
                        .drag_over::<Dragged>(move |style, _, _, _| style.text_color(destructive))
                        .on_drop(cx.listener(|desk, dragged: &Dragged, window, cx| {
                            cx.stop_propagation();
                            desk.discard(dragged, window, cx);
                        }))
                        .child(Icon::new(Glyph::Trash)),
                )
                .into_any_element(),
        )
    }

    fn discard(&mut self, dragged: &Dragged, window: &mut Window, cx: &mut Context<Self>) {
        self.hover = None;
        match dragged {
            Dragged::Terminal(selection) => {
                let found = store(cx)
                    .workspaces
                    .read(cx)
                    .terminal(&selection.machine, &selection.id)
                    .map(|(_, terminal)| terminal.clone());
                if let Some(terminal) = found {
                    dialogs::close_terminal(&selection.machine, &terminal, window, cx);
                }
            }
            Dragged::Tile { id, view } => self.close(id, view.as_ref(), window, cx),
            Dragged::Note { .. } => {}
        }
        cx.notify();
    }

    fn empty(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let page = self
            .empty
            .get_or_insert_with(|| {
                let desk = cx.entity().downgrade();
                cx.new(|cx| StartPage::new(desk, String::new(), window, cx))
            })
            .clone();
        page.into_any_element()
    }
}

impl Render for Desk {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let layout: Layout = self.board.layout().clone();
        let root = match &layout.root {
            Some(root) => self.node(root, &[], window, cx),
            None => self.empty(window, cx),
        };
        let bounds = self.bounds.unwrap_or_default();
        let me = cx.entity();

        div()
            .id("desk")
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .track_focus(&self.focus)
            .child(
                canvas(
                    move |bounds, _, cx| {
                        me.update(cx, |desk, _| desk.bounds = Some(bounds));
                    },
                    |_, (), _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .on_drag_move(cx.listener(|desk, event: &DragMoveEvent<Dragged>, _, cx| {
                let dragged = event.drag(cx).clone();
                desk.drag_move(&dragged, event.event.position, cx);
            }))
            .on_drop(cx.listener(|desk, dragged: &Dragged, window, cx| {
                desk.drop_dragged(dragged, window, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|desk, _, _, cx| desk.end_drag(cx)),
            )
            .child(root)
            .children(self.overlay(bounds, cx))
            .children(self.trash(cx))
    }
}
