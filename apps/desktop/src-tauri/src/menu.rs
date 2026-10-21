use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

pub fn install(app: &tauri::App) -> tauri::Result<()> {
    const CLOSE_TILE: &str = "close-tile";
    const CLOSE_WINDOW: &str = "close-window";

    let handle = app.handle();
    let info = handle.package_info();
    let about = AboutMetadata {
        name: Some(info.name.clone()),
        version: Some(info.version.to_string()),
        ..Default::default()
    };

    let menu = Menu::with_items(
        handle,
        &[
            &Submenu::with_items(
                handle,
                info.name.clone(),
                true,
                &[
                    &PredefinedMenuItem::about(handle, None, Some(about))?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::services(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::hide(handle, None)?,
                    &PredefinedMenuItem::hide_others(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, None)?,
                ],
            )?,
            &Submenu::with_items(
                handle,
                "File",
                true,
                &[
                    &MenuItem::with_id(handle, CLOSE_TILE, "Close Tile", true, Some("Cmd+W"))?,
                    &MenuItem::with_id(
                        handle,
                        CLOSE_WINDOW,
                        "Close Window",
                        true,
                        Some("Cmd+Shift+W"),
                    )?,
                ],
            )?,
            &Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None)?,
                    &PredefinedMenuItem::redo(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None)?,
                    &PredefinedMenuItem::copy(handle, None)?,
                    &PredefinedMenuItem::paste(handle, None)?,
                    &PredefinedMenuItem::select_all(handle, None)?,
                ],
            )?,
            &Submenu::with_items(
                handle,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(handle, None)?],
            )?,
            &Submenu::with_items(
                handle,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, None)?,
                    &PredefinedMenuItem::maximize(handle, None)?,
                ],
            )?,
        ],
    )?;
    app.set_menu(menu)?;

    app.on_menu_event(|app, event| {
        let id = event.id().as_ref();
        if id == CLOSE_TILE {
            if let Err(error) = app.emit(CLOSE_TILE, ()) {
                tracing::warn!(%error, "could not ask the window to close a tile");
            }
        } else if id == CLOSE_WINDOW {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(error) = window.close() {
                    tracing::warn!(%error, "could not close the window");
                }
            }
        }
    });

    Ok(())
}
