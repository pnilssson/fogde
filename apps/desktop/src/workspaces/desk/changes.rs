use gpui_kit::{App, Context, Entity, Global, WeakEntity, Window};

use super::Desk;
use crate::workspaces::tiles::tile::View;

struct Current(WeakEntity<Desk>);

impl Global for Current {}

pub fn current(cx: &App) -> Option<Entity<Desk>> {
    cx.try_global::<Current>()
        .and_then(|current| current.0.upgrade())
}

pub(super) fn register(desk: WeakEntity<Desk>, cx: &mut App) {
    cx.set_global(Current(desk));
}

impl Desk {
    pub fn deleted(
        &mut self,
        machine: &str,
        id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let gone = View::workspace(machine, id);
        self.edit(window, cx, |board| {
            board.leave(machine, id);
            board.forget(&gone);
        });
        self.refresh(window, cx);
    }

    pub fn worktree_removed(&mut self, gone: &View, window: &mut Window, cx: &mut Context<Self>) {
        let home = gone.home();
        let here = self.board.view() == Some(gone);
        self.edit(window, cx, |board| {
            if here {
                board.select(home);
            }
            board.forget(gone);
        });
    }

    pub fn reveal(&mut self, view: View, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.board.select(view);
        self.refresh(window, cx);
        self.activate(id, window, cx);
    }

    pub fn follow_rename(
        &mut self,
        machine: &str,
        from: &str,
        to: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit(window, cx, |board| board.follow(machine, from, to));
    }
}
