use gpui_kit::component::{ActiveTheme, WindowExt, h_flex, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::keymap::{
    CloseTile, MoveDown, MoveLeft, MoveRight, MoveUp, NewTile, NextWaiting, Tile1, Tile2, Tile3,
    Tile4, Tile5, Tile6, Tile7, Tile8, Tile9, ToggleSidebar, WORKBENCH,
};
use crate::sidebar::Sidebar;
use crate::state::store;
use crate::status_bar;
use crate::theme;
use crate::usage::Meters;
use crate::workspaces::desk::Desk;
use crate::workspaces::dialogs;
use crate::workspaces::selection::{Group, folder_of, next_waiting};
use crate::workspaces::tiles::geometry::Side;
use crate::workspaces::tiles::tile::Tile;

pub const NARROW: f32 = 768.;

pub struct Workbench {
    desk: Entity<Desk>,
    sidebar: Entity<Sidebar>,
    meters: Entity<Meters>,
    open: bool,
}

impl Workbench {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let desk = cx.new(|cx| Desk::new(window, cx));
        let sidebar = cx.new(|cx| Sidebar::new(desk.clone(), cx));
        let meters = cx.new(Meters::new);
        Self {
            desk,
            sidebar,
            meters,
            open: true,
        }
    }

    pub fn sidebar_open(&self, window: &Window) -> bool {
        self.open && !narrow(window)
    }

    pub fn toggle_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if narrow(window) {
            let sidebar = self.sidebar.clone();
            window.open_sheet(cx, move |sheet, _, _| {
                sheet.size(px(256.)).child(sidebar.clone())
            });
            return;
        }
        self.open = !self.open;
        cx.notify();
    }

    fn next_waiting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = store(cx);
        let workspaces = state.workspaces.read(cx);
        let machines: Vec<_> = state
            .machines
            .read(cx)
            .connected()
            .map(|machine| machine.id.clone())
            .collect();
        let groups: Vec<Group> = machines
            .iter()
            .map(|machine| Group {
                machine,
                workspaces: workspaces.cache(machine).and_then(|cache| cache.list()),
            })
            .collect();
        let active = self.desk.read(cx).board().active_terminal();
        let Some(next) = next_waiting(&groups, active.as_ref()) else {
            return;
        };
        self.desk.update(cx, |desk, cx| {
            desk.select_and_show(next.view, Some(next.terminal), window, cx);
        });
    }

    fn close_tile(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let desk = self.desk.read(cx);
        let Some(tile) = desk.board().layout().active_tile().cloned() else {
            return;
        };
        match tile {
            Tile::Terminal { machine, id } => {
                let found = store(cx)
                    .workspaces
                    .read(cx)
                    .terminal(&machine, &id)
                    .map(|(_, terminal)| terminal.clone());
                if let Some(terminal) = found {
                    dialogs::close_terminal(&machine, &terminal, window, cx);
                }
            }
            tile => self
                .desk
                .update(cx, |desk, cx| desk.close(&tile.id(), None, window, cx)),
        }
    }

    fn new_tile(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let desk = self.desk.read(cx);
        let Some(view) = desk.view().cloned() else {
            return;
        };
        let Some(workspace) = desk.workspace(cx).filter(|workspace| !workspace.missing) else {
            return;
        };
        let folder = folder_of(workspace, &view);
        self.desk
            .update(cx, |desk, cx| desk.open_start(view, &folder, window, cx));
    }

    fn move_tile(&mut self, side: Side, window: &mut Window, cx: &mut Context<Self>) {
        self.desk
            .update(cx, |desk, cx| desk.move_active(side, window, cx));
    }

    fn focus_tile(&mut self, number: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.desk
            .update(cx, |desk, cx| desk.focus_number(number, window, cx));
    }
}

fn narrow(window: &Window) -> bool {
    f32::from(window.viewport_size().width) < NARROW
}

impl Render for Workbench {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.sidebar_open(window);
        let tokens = theme::tokens(cx);

        v_flex()
            .key_context(WORKBENCH)
            .size_full()
            .min_h_0()
            .on_action(
                cx.listener(|this, _: &NextWaiting, window, cx| this.next_waiting(window, cx)),
            )
            .on_action(cx.listener(|this, _: &CloseTile, window, cx| this.close_tile(window, cx)))
            .on_action(cx.listener(|this, _: &NewTile, window, cx| this.new_tile(window, cx)))
            .on_action(cx.listener(|this, _: &MoveLeft, window, cx| {
                this.move_tile(Side::Left, window, cx);
            }))
            .on_action(cx.listener(|this, _: &MoveRight, window, cx| {
                this.move_tile(Side::Right, window, cx);
            }))
            .on_action(
                cx.listener(|this, _: &MoveUp, window, cx| this.move_tile(Side::Top, window, cx)),
            )
            .on_action(cx.listener(|this, _: &MoveDown, window, cx| {
                this.move_tile(Side::Bottom, window, cx);
            }))
            .on_action(cx.listener(|this, _: &Tile1, window, cx| this.focus_tile(1, window, cx)))
            .on_action(cx.listener(|this, _: &Tile2, window, cx| this.focus_tile(2, window, cx)))
            .on_action(cx.listener(|this, _: &Tile3, window, cx| this.focus_tile(3, window, cx)))
            .on_action(cx.listener(|this, _: &Tile4, window, cx| this.focus_tile(4, window, cx)))
            .on_action(cx.listener(|this, _: &Tile5, window, cx| this.focus_tile(5, window, cx)))
            .on_action(cx.listener(|this, _: &Tile6, window, cx| this.focus_tile(6, window, cx)))
            .on_action(cx.listener(|this, _: &Tile7, window, cx| this.focus_tile(7, window, cx)))
            .on_action(cx.listener(|this, _: &Tile8, window, cx| this.focus_tile(8, window, cx)))
            .on_action(cx.listener(|this, _: &Tile9, window, cx| this.focus_tile(9, window, cx)))
            .on_action(
                cx.listener(|this, _: &ToggleSidebar, window, cx| this.toggle_sidebar(window, cx)),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .when(open, |row| {
                        row.child(
                            div()
                                .w(px(256.))
                                .h_full()
                                .flex_shrink_0()
                                .child(self.sidebar.clone()),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .bg(tokens.surface[2])
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .child(self.desk.clone()),
                    ),
            )
            .child(status_bar::render(&self.meters, cx))
    }
}

pub fn toggle(window: &mut Window, cx: &mut App) {
    window.dispatch_action(Box::new(ToggleSidebar), cx);
}
