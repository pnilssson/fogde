use gpui_kit::component::input::{Copy, Cut, Paste, Redo, SelectAll, Undo};
use gpui_kit::{App, KeyBinding, Menu, MenuItem, OsAction, SystemMenuType, Window, actions};

actions!(
    fogde,
    [
        OpenPalette,
        NextWaiting,
        CloseTile,
        NewTile,
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        ToggleSidebar,
        CloseWindow,
        Hide,
        HideOthers,
        Quit,
        Fullscreen,
        Minimize,
        Zoom,
        Tile1,
        Tile2,
        Tile3,
        Tile4,
        Tile5,
        Tile6,
        Tile7,
        Tile8,
        Tile9,
    ]
);

pub const WORKBENCH: &str = "Workbench";

#[cfg(target_os = "macos")]
macro_rules! chord {
    ($key:literal) => {
        concat!("cmd-", $key)
    };
}

#[cfg(not(target_os = "macos"))]
macro_rules! chord {
    ($key:literal) => {
        concat!("ctrl-shift-", $key)
    };
}

pub fn init(cx: &mut App) {
    bind(cx);
    handle(cx);
    menus(cx);
}

fn bind(cx: &mut App) {
    crate::notes::live::bindings(cx);
    let workbench = Some(WORKBENCH);
    let sidebar = if cfg!(target_os = "macos") {
        "cmd-b"
    } else {
        "ctrl-b"
    };
    cx.bind_keys([
        KeyBinding::new(chord!("k"), OpenPalette, None),
        KeyBinding::new(chord!("g"), NextWaiting, workbench),
        KeyBinding::new(chord!("w"), CloseTile, workbench),
        KeyBinding::new(chord!("t"), NewTile, workbench),
        KeyBinding::new(chord!("alt-left"), MoveLeft, workbench),
        KeyBinding::new(chord!("alt-right"), MoveRight, workbench),
        KeyBinding::new(chord!("alt-up"), MoveUp, workbench),
        KeyBinding::new(chord!("alt-down"), MoveDown, workbench),
        KeyBinding::new(chord!("1"), Tile1, workbench),
        KeyBinding::new(chord!("2"), Tile2, workbench),
        KeyBinding::new(chord!("3"), Tile3, workbench),
        KeyBinding::new(chord!("4"), Tile4, workbench),
        KeyBinding::new(chord!("5"), Tile5, workbench),
        KeyBinding::new(chord!("6"), Tile6, workbench),
        KeyBinding::new(chord!("7"), Tile7, workbench),
        KeyBinding::new(chord!("8"), Tile8, workbench),
        KeyBinding::new(chord!("9"), Tile9, workbench),
        KeyBinding::new(sidebar, ToggleSidebar, Some("Workbench && !Terminal")),
    ]);

    let (bold, italic) = if cfg!(target_os = "macos") {
        ("cmd-b", "cmd-i")
    } else {
        ("ctrl-b", "ctrl-i")
    };
    cx.bind_keys([
        KeyBinding::new(
            bold,
            crate::notes::editor::Bold,
            Some(crate::notes::editor::CONTEXT),
        ),
        KeyBinding::new(
            italic,
            crate::notes::editor::Italic,
            Some(crate::notes::editor::CONTEXT),
        ),
    ]);

    if cfg!(target_os = "macos") {
        cx.bind_keys([
            KeyBinding::new("cmd-shift-w", CloseWindow, None),
            KeyBinding::new("cmd-h", Hide, None),
            KeyBinding::new("cmd-alt-h", HideOthers, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-ctrl-f", Fullscreen, None),
            KeyBinding::new("cmd-m", Minimize, None),
            KeyBinding::new(
                "cmd-c",
                crate::terminals::Copy,
                Some(crate::terminals::CONTEXT),
            ),
            KeyBinding::new(
                "cmd-v",
                crate::terminals::Paste,
                Some(crate::terminals::CONTEXT),
            ),
        ]);
    }
}

fn handle(cx: &mut App) {
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &OpenPalette, cx| {
        if let Some(window) = cx.active_window() {
            window
                .update(cx, |_, window, cx| open_palette(window, cx))
                .ok();
        }
    });
    cx.on_action(|_: &CloseWindow, cx| {
        if let Some(window) = cx.active_window() {
            window
                .update(cx, |_, window, _| window.remove_window())
                .ok();
        }
    });
    cx.on_action(|_: &Fullscreen, cx| with_window(cx, Window::toggle_fullscreen));
    cx.on_action(|_: &Minimize, cx| with_window(cx, Window::minimize_window));
    cx.on_action(|_: &Zoom, cx| with_window(cx, Window::zoom_window));
}

fn menus(cx: &mut App) {
    if cfg!(target_os = "macos") {
        cx.set_menus([
            Menu::new("Fogde").items([
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("Hide Fogde", Hide),
                MenuItem::action("Hide Others", HideOthers),
                MenuItem::separator(),
                MenuItem::action("Quit Fogde", Quit),
            ]),
            Menu::new("File").items([
                MenuItem::action("Close Tile", CloseTile),
                MenuItem::action("Close Window", CloseWindow),
            ]),
            Menu::new("Edit").items([
                MenuItem::os_action("Undo", Undo, OsAction::Undo),
                MenuItem::os_action("Redo", Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("Cut", Cut, OsAction::Cut),
                MenuItem::os_action("Copy", Copy, OsAction::Copy),
                MenuItem::os_action("Paste", Paste, OsAction::Paste),
                MenuItem::os_action("Select All", SelectAll, OsAction::SelectAll),
            ]),
            Menu::new("View").items([MenuItem::action("Enter Full Screen", Fullscreen)]),
            Menu::new("Window").items([
                MenuItem::action("Minimize", Minimize),
                MenuItem::action("Zoom", Zoom),
            ]),
        ]);
    }
}

fn with_window(cx: &mut App, run: fn(&Window)) {
    if let Some(window) = cx.active_window() {
        window.update(cx, |_, window, _| run(window)).ok();
    }
}

pub fn open_palette(window: &mut Window, cx: &mut App) {
    crate::palette::toggle(window, cx);
}
