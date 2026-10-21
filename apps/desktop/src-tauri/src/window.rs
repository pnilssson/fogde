use protocol::theme::Material;
use protocol::{Frame, Platform};
use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn open(app: &App) -> tauri::Result<()> {
    let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Fogde")
        .inner_size(1200.0, 800.0)
        .center()
        .min_inner_size(720.0, 480.0)
        .disable_drag_drop_handler();

    #[cfg(windows)]
    let (builder, frame) = {
        let translucent = windows_version::OsVersion::current().build >= 22_523;
        let builder = builder.decorations(false);
        let builder = if translucent {
            builder.transparent(true).effects(glass::effects())
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

        let builder = builder
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true)
            .transparent(true)
            .effects(glass::effects());
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

#[cfg(any(windows, target_os = "macos"))]
mod glass {
    use tauri::utils::config::WindowEffectsConfig;
    use tauri::window::{Effect, EffectsBuilder};

    #[cfg(windows)]
    pub fn effects() -> WindowEffectsConfig {
        EffectsBuilder::new().effect(Effect::Acrylic).build()
    }

    #[cfg(target_os = "macos")]
    pub fn effects() -> WindowEffectsConfig {
        EffectsBuilder::new()
            .effects([Effect::LiquidGlassRegular, Effect::Sidebar])
            .build()
    }
}

#[cfg(any(windows, target_os = "macos"))]
#[tauri::command]
pub async fn material(
    window: tauri::WebviewWindow,
    frame: tauri::State<'_, Frame>,
    material: Material,
) -> tauri::Result<()> {
    if !frame.translucent {
        return Ok(());
    }

    window.set_effects(match material {
        Material::Translucent => Some(glass::effects()),
        Material::Solid => None,
    })
}

#[cfg(not(any(windows, target_os = "macos")))]
#[tauri::command]
pub async fn material(_material: Material) -> tauri::Result<()> {
    Ok(())
}
