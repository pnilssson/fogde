use super::{Content, Desk, Hover};
use crate::state::store;
use crate::workspaces::tiles::geometry::{Rect, Side};
use crate::workspaces::tiles::tile::{Selection, View};
use crate::workspaces::tiles::{Board, Dropped};
use gpui_kit::{Context, Focusable, Window};

impl Desk {
    pub fn select(&mut self, view: View, window: &mut Window, cx: &mut Context<Self>) {
        self.board.select(view);
        self.refresh(window, cx);
    }

    pub fn select_and_show(
        &mut self,
        view: View,
        terminal: Option<Selection>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.board.select(view);
        if let Some(terminal) = terminal {
            self.board.show(terminal);
        }
        self.refresh(window, cx);
        self.focus_active(window, cx);
    }

    pub fn activate(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.board.activate(id);
        self.ensure_contents(window, cx);
        self.focus_active(window, cx);
        self.changed(cx);
    }

    pub fn focus_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(active) = self.board.active() else {
            window.focus(&self.focus, cx);
            return;
        };
        let handle = match self.contents.get(active) {
            Some(Content::Terminal(view)) => view.focus_handle(cx),
            Some(Content::Start(view)) => view.focus_handle(cx),
            Some(Content::Notes(view)) => view.focus_handle(cx),
            None => self.focus.clone(),
        };
        window.focus(&handle, cx);
    }

    pub fn edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        edit: impl FnOnce(&mut Board),
    ) {
        edit(&mut self.board);
        self.ensure_contents(window, cx);
        self.focus_active(window, cx);
        self.changed(cx);
    }

    pub fn open_start(
        &mut self,
        view: View,
        folder: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.board.select(view);
        self.edit(window, cx, |board| board.open_start(folder));
    }

    pub fn close(
        &mut self,
        id: &str,
        view: Option<&View>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit(window, cx, |board| board.close(id, view));
    }

    pub fn rects(&self) -> Vec<(String, Rect)> {
        let bounds = self.bounds.unwrap_or_default();
        self.board.layout().rects(Rect {
            x: bounds.origin.x.into(),
            y: bounds.origin.y.into(),
            width: f32::from(bounds.size.width).max(1.),
            height: f32::from(bounds.size.height).max(1.),
        })
    }

    pub fn move_active(&mut self, side: Side, window: &mut Window, cx: &mut Context<Self>) {
        let rects = self.rects();
        self.edit(window, cx, |board| board.move_active(side, &rects));
    }

    pub fn focus_number(&mut self, number: usize, window: &mut Window, cx: &mut Context<Self>) {
        let order = crate::workspaces::tiles::geometry::numbered(&self.rects());
        if let Some(id) = number
            .checked_sub(1)
            .and_then(|index| order.get(index))
            .cloned()
        {
            self.activate(&id, window, cx);
        }
    }

    pub fn dropped(&mut self, dropped: Dropped, cx: &mut Context<Self>) {
        let Some(view) = self.board.view().cloned() else {
            return;
        };
        match dropped {
            Dropped::Placed(selection) => {
                let params = protocol::workspaces::PlaceTerminalParams {
                    terminal: selection.id,
                    workspace: view.id,
                    checkout: view.checkout,
                };
                let machine = selection.machine;
                store(cx)
                    .workspaces
                    .clone()
                    .update(cx, |workspaces, cx| workspaces.place(&machine, params, cx));
            }
            Dropped::Paste(selection) => {
                if let Some(Content::Terminal(terminal)) = self.contents.get(&selection.tile())
                    && let Some(path) = self.hover.as_ref().and_then(Hover::note_path)
                {
                    let text = terminal::bracket_path(&path);
                    terminal.update(cx, |terminal, _| terminal.paste_text(&text));
                }
            }
            Dropped::Moved | Dropped::Nothing => {}
        }
    }
}
