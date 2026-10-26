mod terminal;

use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::{ActiveTheme, Icon, Sizable};
use gpui_kit::{
    AnyElement, AppContext, Context, DragMoveEvent, IntoElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use protocol::terminals::Terminal;
use protocol::workspaces::{MoveTerminalParams, Workspace};

use super::Sidebar;
use super::row::row;
use crate::assets::Glyph;
use crate::state::machines::Machine;
use crate::state::store;
use crate::workspaces::desk::split::Blank;
use crate::workspaces::order::{Edge, before_of, changes};
use crate::workspaces::selection::in_view;
use crate::workspaces::tiles::tile::{Dragged, START, View};

impl Sidebar {
    fn focused(&self, workspace: &Workspace, view: &View, cx: &Context<Self>) -> Option<String> {
        let desk = self.desk.read(cx);
        if desk.view() != Some(view) {
            return None;
        }
        let board = desk.board();
        let focused = board.active()?.to_owned();
        if board.note_tiles(view).iter().any(|(id, _)| *id == focused) {
            return Some(focused);
        }
        if focused == START {
            return board.shared_start(view).map(|_| focused);
        }
        let active = board.active_terminal()?;
        let listed = workspace
            .terminals
            .iter()
            .any(|terminal| terminal.id == active.id && in_view(view, terminal));
        (listed && active.machine == view.machine).then_some(active.id)
    }

    fn arrange(
        machine: &str,
        terminal: &str,
        before: Option<String>,
        siblings: &[String],
        cx: &mut Context<Self>,
    ) {
        if !changes(siblings, terminal, before.as_deref()) {
            return;
        }
        let params = MoveTerminalParams {
            terminal: terminal.to_owned(),
            before,
        };
        let machine = machine.to_owned();
        store(cx).workspaces.clone().update(cx, |workspaces, cx| {
            workspaces.arrange(&machine, params, cx);
        });
    }

    fn drag_over(
        &mut self,
        event: &DragMoveEvent<Dragged>,
        target: &str,
        machine: &str,
        siblings: &[String],
        cx: &mut Context<Self>,
    ) {
        let Dragged::Terminal(moving) = event.drag(cx).clone() else {
            return;
        };
        if !event.bounds.contains(&event.event.position) {
            if self.over.as_ref().is_some_and(|(id, _)| *id == target) {
                self.over = None;
                cx.notify();
            }
            return;
        }
        let valid =
            moving.machine == machine && moving.id != target && siblings.contains(&moving.id);
        let edge = if event.event.position.y < event.bounds.center().y {
            Edge::Above
        } else {
            Edge::Below
        };
        let before = before_of(siblings, &moving.id, target, edge);
        let next = (valid && changes(siblings, &moving.id, before.as_deref()))
            .then(|| (target.to_owned(), edge));
        if self.over != next {
            self.over = next;
            cx.notify();
        }
    }

    pub(super) fn rows(
        &mut self,
        machine: &Machine,
        workspace: &Workspace,
        view: &View,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let focused = self.focused(workspace, view, cx);
        let terminals: Vec<&Terminal> = workspace
            .terminals
            .iter()
            .filter(|terminal| in_view(view, terminal))
            .collect();
        let siblings: Vec<String> = terminals
            .iter()
            .map(|terminal| terminal.id.clone())
            .collect();
        let board = self.desk.read(cx).board();
        let notes = board.note_tiles(view);
        let start = board.shared_start(view);

        let mut rows: Vec<AnyElement> = terminals
            .into_iter()
            .map(|terminal| {
                let active = focused.as_deref() == Some(terminal.id.as_str());
                self.terminal(machine, view, terminal, &siblings, active, cx)
            })
            .collect();

        for (id, title) in notes {
            let active = focused.as_deref() == Some(id.as_str());
            let (desk, close_desk) = (self.desk.clone(), self.desk.clone());
            let (reveal_view, reveal_id) = (view.clone(), id.clone());
            let (close_view, close_id) = (view.clone(), id.clone());
            let dragged = Dragged::Tile {
                id: id.clone(),
                view: Some(view.clone()),
            };
            rows.push(
                row(
                    SharedString::from(format!("note-{id}")),
                    Icon::new(Glyph::Notebook).small().into_any_element(),
                    title,
                    active,
                    true,
                    cx,
                )
                .on_click(move |_, window, cx| {
                    let view = reveal_view.clone();
                    desk.update(cx, |desk, cx| desk.reveal(view, &reveal_id, window, cx));
                })
                .on_drag(dragged, |_, _, _, cx| cx.new(|_| Blank))
                .context_menu(move |menu, _, _| {
                    let (desk, view, id) =
                        (close_desk.clone(), close_view.clone(), close_id.clone());
                    menu.item(PopupMenuItem::new("Close").on_click(move |_, window, cx| {
                        desk.update(cx, |desk, cx| desk.close(&id, Some(&view), window, cx));
                    }))
                })
                .into_any_element(),
            );
        }

        if let Some(folder) = start {
            let active = focused.as_deref() == Some(START);
            let (desk, close_desk) = (self.desk.clone(), self.desk.clone());
            let (open_view, close_view) = (view.clone(), view.clone());
            let muted = cx.theme().muted_foreground;
            rows.push(
                row(
                    SharedString::from(format!("start-{}", view.key())),
                    Icon::new(Glyph::PlusCircle)
                        .small()
                        .text_color(muted)
                        .into_any_element(),
                    "New tile",
                    active,
                    true,
                    cx,
                )
                .on_click(move |_, window, cx| {
                    let view = open_view.clone();
                    desk.update(cx, |desk, cx| desk.open_start(view, &folder, window, cx));
                })
                .context_menu(move |menu, _, _| {
                    let (desk, view) = (close_desk.clone(), close_view.clone());
                    menu.item(PopupMenuItem::new("Close").on_click(move |_, window, cx| {
                        desk.update(cx, |desk, cx| desk.close(START, Some(&view), window, cx));
                    }))
                })
                .into_any_element(),
            );
        }

        rows
    }
}
