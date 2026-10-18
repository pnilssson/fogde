#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod window;

use protocol::{Connection, Frame};
use tauri::State;

#[tauri::command]
async fn connection(connection: State<'_, Connection>) -> Result<Connection, ()> {
    Ok(connection.inner().clone())
}

#[tauri::command]
async fn frame(frame: State<'_, Frame>) -> Result<Frame, ()> {
    Ok(*frame)
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let host = tauri::async_runtime::block_on(host::start())?;

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(host)
        .setup(|app| Ok(window::open(app)?))
        .invoke_handler(tauri::generate_handler![connection, frame])
        .run(tauri::generate_context!())?;

    Ok(())
}
