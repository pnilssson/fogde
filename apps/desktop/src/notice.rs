use std::fmt::Display;

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, AppContext, Entity, EventEmitter, Global, SharedString, Window};

#[derive(Clone)]
pub enum Notice {
    Error {
        title: SharedString,
        message: SharedString,
    },
    Warning {
        title: SharedString,
        message: Option<SharedString>,
    },
}

pub struct Notices;

impl EventEmitter<Notice> for Notices {}

struct Hub(Entity<Notices>);

impl Global for Hub {}

pub fn init(cx: &mut App) {
    let notices = cx.new(|_| Notices);
    cx.set_global(Hub(notices));
}

pub fn notices(cx: &App) -> Entity<Notices> {
    cx.global::<Hub>().0.clone()
}

pub fn show(notice: Notice, cx: &mut App) {
    notices(cx).update(cx, |_, cx| cx.emit(notice));
}

pub fn error(title: impl Into<SharedString>, error: &impl Display, cx: &mut App) {
    show(
        Notice::Error {
            title: title.into(),
            message: error.to_string().into(),
        },
        cx,
    );
}

pub fn warning(title: impl Into<SharedString>, message: Option<SharedString>, cx: &mut App) {
    show(
        Notice::Warning {
            title: title.into(),
            message,
        },
        cx,
    );
}

pub fn present(notice: &Notice, window: &mut Window, cx: &mut App) {
    let notification = match notice.clone() {
        Notice::Error { title, message } => {
            Notification::error(message).title(title).autohide(false)
        }
        Notice::Warning { title, message } => match message {
            Some(message) => Notification::warning(message).title(title),
            None => Notification::warning(title),
        },
    };
    window.push_notification(notification, cx);
}
