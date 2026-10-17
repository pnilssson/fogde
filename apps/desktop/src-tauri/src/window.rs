use protocol::{Frame, Platform};
use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn open(app: &App) -> tauri::Result<()> {
    let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Fogde")
        .inner_size(1200.0, 800.0)
        .min_inner_size(720.0, 480.0);

    #[cfg(windows)]
    let (builder, frame) = {
        use tauri::window::{Effect, EffectsBuilder};

        let translucent = windows_version::OsVersion::current().build >= 22_523;
        let builder = builder.decorations(false);
        let builder = if translucent {
            builder
                .transparent(true)
                .effects(EffectsBuilder::new().effect(Effect::Acrylic).build())
        } else {
            builder
        };
        let frame = Frame {
            platform: Platform::Windows,
            translucent,
        };
        (builder, frame)
    };

    #[cfg(target_os = "macos")]
    let (builder, frame) = {
        use tauri::TitleBarStyle;
        use tauri::window::{Effect, EffectsBuilder};

        let effects = EffectsBuilder::new()
            .effects([Effect::LiquidGlassRegular, Effect::Sidebar])
            .build();
        let builder = builder
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true)
            .transparent(true)
            .effects(effects);
        let frame = Frame {
            platform: Platform::Macos,
            translucent: true,
        };
        (builder, frame)
    };

    #[cfg(not(any(windows, target_os = "macos")))]
    let frame = Frame {
        platform: Platform::Linux,
        translucent: false,
    };

    app.manage(frame);
    builder.build()?;
    Ok(())
}
