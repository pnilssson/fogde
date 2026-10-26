#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod attention;
mod folders;
mod keymap;
mod notes;
mod notice;
mod num;
mod onboarding;
mod palette;
mod paths;
mod persist;
mod runtime;
mod servers;
mod settings;
mod sidebar;
mod state;
mod status_bar;
mod terminals;
mod theme;
mod updates;
mod usage;
mod window;
mod workbench;
mod workspaces;

use client::Client;

fn main() -> anyhow::Result<()> {
    if let Err(error) = host::log("desktop", cfg!(debug_assertions)) {
        tracing::warn!(%error, "could not open the log file");
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let connection = runtime.block_on(host::start(servers::binaries()))?;
    let handle = runtime.handle().clone();

    gpui_kit::application()
        .with_assets(assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            keymap::init(cx);
            runtime::init(handle.clone(), cx);
            notice::init(cx);
            updates::init(cx);
            persist::init(persist::load(), cx);
            let material = persist::saved(cx).material.unwrap_or_default();
            theme::init(window::translucent(), material, cx);

            let own = {
                let _guard = handle.enter();
                Client::local(connection.clone())
            };
            state::init(own, connection.clone(), cx);
            window::open(cx);
        });

    drop(runtime);
    Ok(())
}
