use super::{Desk, DeskEvent};
use crate::persist;
use gpui_kit::Context;
use std::collections::BTreeMap;

impl Desk {
    pub(super) fn changed(&mut self, cx: &mut Context<Self>) {
        let view = self.board.view().cloned();
        let layouts: BTreeMap<String, serde_json::Value> = self
            .board
            .saved()
            .iter()
            .filter_map(|(key, layout)| {
                serde_json::to_value(layout)
                    .ok()
                    .map(|value| (key.clone(), value))
            })
            .collect();
        let closed = self.board.closed().clone();
        let unchanged = {
            let saved = persist::saved(cx);
            saved.layouts == layouts
                && saved.closed == closed
                && saved.workspace
                    == view
                        .as_ref()
                        .map(|view| (view.machine.clone(), view.id.clone()))
                && saved.checkout == view.as_ref().and_then(|view| view.checkout.clone())
        };
        if !unchanged {
            persist::update(cx, |saved| {
                saved.layouts = layouts;
                saved.closed = closed;
                saved.workspace = view
                    .as_ref()
                    .map(|view| (view.machine.clone(), view.id.clone()));
                saved.checkout = view.and_then(|view| view.checkout);
            });
        }

        self.view_changed(cx);
        cx.emit(DeskEvent::Changed);
        cx.notify();
    }

    pub(super) fn view_changed(&mut self, cx: &mut Context<Self>) {
        let viewing = self
            .focused_window
            .then(|| self.board.active_terminal())
            .flatten();
        self.viewing.set(viewing, cx);
    }
}
