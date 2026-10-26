use crate::sidebar::Sidebar;
use crate::sidebar::row::row;
use crate::state::machines::Machine;
use crate::state::store;
use crate::workspaces::desk::split::Blank;
use crate::workspaces::dialogs;
use crate::workspaces::order::{Edge, before_of};
use crate::workspaces::tiles::tile::{Dragged, Selection, View};
use crate::workspaces::{status_dot, terminal_icon};
use gpui_kit::component::menu::{ContextMenuExt, PopupMenu, PopupMenuItem};
use gpui_kit::{
    AnyElement, AppContext, Context, DragMoveEvent, Entity, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
    px,
};
use protocol::terminals::Terminal;

impl Sidebar {
    pub(super) fn terminal(
        &mut self,
        machine: &Machine,
        view: &View,
        terminal: &Terminal,
        siblings: &[String],
        active: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selection = Selection::new(machine.id.to_string(), terminal.id.clone());
        let over = self
            .over
            .as_ref()
            .filter(|(id, _)| *id == terminal.id)
            .map(|(_, edge)| *edge);
        let ring = crate::theme::tokens(cx).foreground;
        let connected = store(cx).machines.read(cx).state(&machine.id) == client::State::Connected;
        let closable = connected
            && !store(cx)
                .workspaces
                .read(cx)
                .closing(&machine.id, &terminal.id);

        let (desk, show_view, show) = (self.desk.clone(), view.clone(), selection.clone());
        let (move_siblings, move_machine, target) = (
            siblings.to_vec(),
            machine.id.to_string(),
            terminal.id.clone(),
        );
        let (drop_siblings, drop_machine, drop_target) = (
            siblings.to_vec(),
            machine.id.to_string(),
            terminal.id.clone(),
        );
        let (menu_siblings, menu_machine, menu_terminal) =
            (siblings.to_vec(), machine.id.to_string(), terminal.clone());
        let me = cx.entity();

        row(
            SharedString::from(format!("terminal-{}-{}", machine.id, terminal.id)),
            terminal_icon(terminal.agent, cx),
            terminal.name.clone(),
            active,
            true,
            cx,
        )
        .relative()
        .children(status_dot(terminal.status, cx))
        .when_some(over, |row, edge| {
            row.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .h(px(2.))
                    .rounded_full()
                    .bg(ring)
                    .when(edge == Edge::Above, |line| line.top(px(-1.)))
                    .when(edge == Edge::Below, |line| line.bottom(px(-1.))),
            )
        })
        .on_click(move |_, window, cx| {
            let (view, selection) = (show_view.clone(), show.clone());
            desk.update(cx, |desk, cx| {
                desk.select_and_show(view, Some(selection), window, cx);
            });
        })
        .on_drag(Dragged::Terminal(selection.clone()), |_, _, _, cx| {
            cx.new(|_| Blank)
        })
        .on_drag_move(
            cx.listener(move |this, event: &DragMoveEvent<Dragged>, _, cx| {
                this.drag_over(event, &target, &move_machine, &move_siblings, cx);
            }),
        )
        .on_drop(cx.listener(move |this, dragged: &Dragged, _, cx| {
            let Dragged::Terminal(moving) = dragged else {
                return;
            };
            let Some((_, edge)) = this.over.take().filter(|(id, _)| *id == drop_target) else {
                return;
            };
            cx.stop_propagation();
            let before = before_of(&drop_siblings, &moving.id, &drop_target, edge);
            Self::arrange(&drop_machine, &moving.id, before, &drop_siblings, cx);
        }))
        .context_menu(move |menu, _, _| {
            terminal_menu(
                menu,
                &me,
                &menu_machine,
                &menu_terminal,
                &menu_siblings,
                closable,
            )
        })
        .into_any_element()
    }
}

pub(super) fn terminal_menu(
    menu: PopupMenu,
    me: &Entity<Sidebar>,
    machine: &str,
    terminal: &Terminal,
    siblings: &[String],
    closable: bool,
) -> PopupMenu {
    let (up, down) = (me.clone(), me.clone());
    let index = siblings
        .iter()
        .position(|id| *id == terminal.id)
        .unwrap_or_default();
    let (up_siblings, down_siblings) = (siblings.to_vec(), siblings.to_vec());
    let (up_machine, down_machine) = (machine.to_owned(), machine.to_owned());
    let (up_id, down_id) = (terminal.id.clone(), terminal.id.clone());
    let (close_machine, close_terminal) = (machine.to_owned(), terminal.clone());
    let last = siblings.len().saturating_sub(1);
    menu.item(
        PopupMenuItem::new("Move up")
            .disabled(index == 0)
            .on_click(move |_, _, cx| {
                let before = up_siblings.get(index.saturating_sub(1)).cloned();
                up.update(cx, |_, cx| {
                    Sidebar::arrange(&up_machine, &up_id, before, &up_siblings, cx);
                });
            }),
    )
    .item(
        PopupMenuItem::new("Move down")
            .disabled(index >= last)
            .on_click(move |_, _, cx| {
                let before = down_siblings.get(index + 2).cloned();
                down.update(cx, |_, cx| {
                    Sidebar::arrange(&down_machine, &down_id, before, &down_siblings, cx);
                });
            }),
    )
    .separator()
    .item(
        PopupMenuItem::new("Close")
            .disabled(!closable)
            .on_click(move |_, window, cx| {
                dialogs::close_terminal(&close_machine, &close_terminal, window, cx);
            }),
    )
}
