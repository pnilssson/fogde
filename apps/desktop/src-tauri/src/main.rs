#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;

use std::path::Path;

use protocol::{Connection, Frame};
use tauri::{Manager, State};

#[tauri::command]
async fn connection(connection: State<'_, Connection>) -> Result<Connection, ()> {
    Ok(connection.inner().clone())
}

#[tauri::command]
async fn frame(frame: State<'_, Frame>) -> Result<Frame, ()> {
    Ok(*frame)
}

fn binaries(app: &tauri::App) -> tauri::Result<host::Binaries> {
    if cfg!(debug_assertions) {
        return Ok(host::Binaries::built(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target"),
        ));
    }

    Ok(host::Binaries::bundled(
        app.path().resource_dir()?.join("servers"),
    ))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let host = tauri::async_runtime::block_on(host::start(binaries(app)?))?;
            app.manage(host);
            Ok(window::open(app)?)
        })
        .invoke_handler(tauri::generate_handler![connection, frame])
        .run(tauri::generate_context!())?;

    Ok(())
}
