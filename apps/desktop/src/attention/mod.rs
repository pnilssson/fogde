mod notify;
mod sound;

use gpui_kit::{AnyWindowHandle, App, AppContext, Context, Entity, Subscription, Window};
use protocol::terminals::Status;

use crate::state::store;
use crate::state::workspaces::{Activity, Workspaces};
use crate::workspaces::desk;
use crate::workspaces::selection::view_of;
use crate::workspaces::status::needs_you;
use crate::workspaces::tiles::tile::Selection;

pub struct Attention {
    window: AnyWindowHandle,
    _activity: Subscription,
}

fn message(status: Status) -> Option<&'static str> {
    match status {
        Status::Blocked => Some("Needs your input in"),
        Status::Done => Some("Finished in"),
        Status::Idle | Status::Working => None,
    }
}

impl Attention {
    pub fn new(workspaces: &Entity<Workspaces>, window: &Window, cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe(workspaces, |this, _, activity, cx| {
            this.changed(activity, cx);
        });
        Self {
            window: window.window_handle(),
            _activity: subscription,
        }
    }

    fn changed(&mut self, activity: &Activity, cx: &mut Context<Self>) {
        let Activity::State {
            machine,
            state,
            applied,
        } = activity;
        let Some(message) = message(state.status) else {
            return;
        };
        if needs_you(applied.before.status) {
            return;
        }

        let selection = Selection::new(machine.to_string(), state.id.clone());
        let focused = self
            .window
            .update(cx, |_, window, _| window.is_window_active())
            .unwrap_or(false);
        let shown = desk::current(cx).is_some_and(|desk| desk.read(cx).board().shows(&selection));
        if focused && shown {
            return;
        }

        let Some(settings) = store(cx).settings.read(cx).notifications else {
            return;
        };
        if settings.sound {
            sound::chime();
        }
        if settings.enabled {
            let view = view_of(machine, &applied.workspace, Some(&applied.before));
            let clicked = notify::show(
                state.name.clone(),
                format!("{message} {}", applied.workspace.name),
            );
            let handle = self.window;
            cx.spawn(async move |_, cx| {
                if clicked.await.is_ok() {
                    cx.update(|cx| reveal(handle, view, selection, cx));
                }
            })
            .detach();
        }
        self.window
            .update(cx, |_, window, _| window.request_attention())
            .ok();
    }
}

fn reveal(
    handle: AnyWindowHandle,
    view: crate::workspaces::tiles::tile::View,
    selection: Selection,
    cx: &mut App,
) {
    cx.activate(true);
    handle
        .update(cx, |_, window, cx| {
            window.activate_window();
            if let Some(desk) = desk::current(cx) {
                desk.update(cx, |desk, cx| {
                    desk.select_and_show(view, Some(selection), window, cx);
                });
            }
        })
        .ok();
}

pub fn init(window: &Window, cx: &mut App) -> Entity<Attention> {
    let workspaces = store(cx).workspaces.clone();
    cx.new(|cx| Attention::new(&workspaces, window, cx))
}
