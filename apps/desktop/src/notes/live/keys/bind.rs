use super::{
    Backspace, Bottom, CONTEXT, Delete, DeleteHome, DeleteWordLeft, DeleteWordRight, Down, End,
    Enter, Home, Indent, Left, Outdent, PageDown, PageUp, Right, SelectBottom, SelectDown,
    SelectEnd, SelectHome, SelectLeft, SelectRight, SelectTop, SelectUp, SelectWordLeft,
    SelectWordRight, Top, Up, WordLeft, WordRight,
};
use gpui_kit::component::input::{Copy, Cut, Paste, Redo, SelectAll, Undo};
use gpui_kit::{App, KeyBinding};

#[cfg(target_os = "macos")]
const PLATFORM: [(&str, &str); 12] = [
    ("alt-left", "word-left"),
    ("alt-right", "word-right"),
    ("alt-shift-left", "select-word-left"),
    ("alt-shift-right", "select-word-right"),
    ("cmd-left", "home"),
    ("cmd-right", "end"),
    ("cmd-shift-left", "select-home"),
    ("cmd-shift-right", "select-end"),
    ("cmd-up", "top"),
    ("cmd-down", "bottom"),
    ("alt-backspace", "delete-word-left"),
    ("cmd-backspace", "delete-home"),
];

#[cfg(not(target_os = "macos"))]
const PLATFORM: [(&str, &str); 12] = [
    ("ctrl-left", "word-left"),
    ("ctrl-right", "word-right"),
    ("ctrl-shift-left", "select-word-left"),
    ("ctrl-shift-right", "select-word-right"),
    ("ctrl-home", "top"),
    ("ctrl-end", "bottom"),
    ("ctrl-shift-home", "select-top"),
    ("ctrl-shift-end", "select-bottom"),
    ("ctrl-backspace", "delete-word-left"),
    ("ctrl-delete", "delete-word-right"),
    ("ctrl-y", "redo"),
    ("ctrl-shift-z", "redo"),
];

fn binding(keys: &'static str, name: &str) -> Option<KeyBinding> {
    let context = Some(CONTEXT);
    Some(match name {
        "word-left" => KeyBinding::new(keys, WordLeft, context),
        "word-right" => KeyBinding::new(keys, WordRight, context),
        "select-word-left" => KeyBinding::new(keys, SelectWordLeft, context),
        "select-word-right" => KeyBinding::new(keys, SelectWordRight, context),
        "home" => KeyBinding::new(keys, Home, context),
        "end" => KeyBinding::new(keys, End, context),
        "select-home" => KeyBinding::new(keys, SelectHome, context),
        "select-end" => KeyBinding::new(keys, SelectEnd, context),
        "top" => KeyBinding::new(keys, Top, context),
        "bottom" => KeyBinding::new(keys, Bottom, context),
        "select-top" => KeyBinding::new(keys, SelectTop, context),
        "select-bottom" => KeyBinding::new(keys, SelectBottom, context),
        "delete-word-left" => KeyBinding::new(keys, DeleteWordLeft, context),
        "delete-word-right" => KeyBinding::new(keys, DeleteWordRight, context),
        "delete-home" => KeyBinding::new(keys, DeleteHome, context),
        "redo" => KeyBinding::new(keys, Redo, context),
        _ => return None,
    })
}

pub fn bindings(cx: &mut App) {
    let context = Some(CONTEXT);
    let secondary = |mac: &'static str, other: &'static str| {
        if cfg!(target_os = "macos") {
            mac
        } else {
            other
        }
    };
    let mut list = vec![
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("up", Up, context),
        KeyBinding::new("down", Down, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("shift-up", SelectUp, context),
        KeyBinding::new("shift-down", SelectDown, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
        KeyBinding::new("pageup", PageUp, context),
        KeyBinding::new("pagedown", PageDown, context),
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("shift-backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("enter", Enter, context),
        KeyBinding::new("shift-enter", Enter, context),
        KeyBinding::new("tab", Indent, context),
        KeyBinding::new("shift-tab", Outdent, context),
        KeyBinding::new(secondary("cmd-a", "ctrl-a"), SelectAll, context),
        KeyBinding::new(secondary("cmd-c", "ctrl-c"), Copy, context),
        KeyBinding::new(secondary("cmd-x", "ctrl-x"), Cut, context),
        KeyBinding::new(secondary("cmd-v", "ctrl-v"), Paste, context),
        KeyBinding::new(secondary("cmd-z", "ctrl-z"), Undo, context),
        KeyBinding::new(secondary("cmd-shift-z", "ctrl-shift-z"), Redo, context),
    ];
    list.extend(
        PLATFORM
            .iter()
            .filter_map(|(keys, name)| binding(keys, name)),
    );
    cx.bind_keys(list);
}
