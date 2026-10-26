use gpui_kit::component::TitleBar;
use gpui_kit::{
    App, AppContext, Bounds, Pixels, TitlebarOptions, Window, WindowBackgroundAppearance,
    WindowBounds, WindowDecorations, WindowOptions, point, px, size,
};
use protocol::theme::Material;

use crate::app::Fogde;
use crate::persist::{self, Place};
use crate::theme;

pub const BAR: Pixels = px(40.);

pub fn translucent() -> bool {
    #[cfg(windows)]
    {
        windows_version::OsVersion::current().build >= 22_523
    }
    #[cfg(target_os = "macos")]
    {
        true
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        false
    }
}

pub fn background(cx: &App) -> WindowBackgroundAppearance {
    if theme::translucent(cx) {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    }
}

fn fits(place: &Place, cx: &App) -> bool {
    let bounds = Bounds::new(
        point(px(place.x), px(place.y)),
        size(px(place.width), px(place.height)),
    );
    cx.displays().iter().any(|display| {
        let screen = display.bounds();
        screen.contains(&bounds.origin)
            && bounds.size.width <= screen.size.width
            && bounds.size.height <= screen.size.height
    })
}

fn bounds(cx: &App) -> WindowBounds {
    let restored = persist::saved(cx)
        .window
        .filter(|place| fits(place, cx))
        .map(|place| {
            let bounds = Bounds::new(
                point(px(place.x), px(place.y)),
                size(px(place.width), px(place.height)),
            );
            if place.maximized {
                WindowBounds::Maximized(bounds)
            } else {
                WindowBounds::Windowed(bounds)
            }
        });

    restored.unwrap_or_else(|| {
        WindowBounds::Windowed(Bounds::centered(None, size(px(1200.), px(800.)), cx))
    })
}

pub fn options(cx: &App) -> WindowOptions {
    let titlebar = if cfg!(target_os = "linux") {
        None
    } else {
        Some(TitlebarOptions {
            title: Some("Fogde".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(13.), px(13.))),
        })
    };

    WindowOptions {
        window_bounds: Some(bounds(cx)),
        titlebar,
        app_owns_titlebar_drag: !cfg!(target_os = "linux"),
        window_min_size: Some(size(px(720.), px(480.))),
        window_background: background(cx),
        window_decorations: Some(WindowDecorations::Server),
        app_id: Some("dev.fogde.app".into()),
        ..TitleBar::window_options()
    }
}

pub fn open(cx: &mut App) {
    let options = options(cx);
    if let Err(error) = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| Fogde::new(window, cx))
    }) {
        tracing::error!(%error, "could not open the window");
        cx.quit();
    }
}

pub fn remember(window: &Window, cx: &mut App) {
    let (bounds, maximized) = match window.window_bounds() {
        WindowBounds::Windowed(bounds) => (bounds, false),
        WindowBounds::Maximized(bounds) | WindowBounds::Fullscreen(bounds) => (bounds, true),
    };
    let place = Place {
        x: bounds.origin.x.into(),
        y: bounds.origin.y.into(),
        width: bounds.size.width.into(),
        height: bounds.size.height.into(),
        maximized,
    };
    if persist::saved(cx).window != Some(place) {
        persist::update(cx, |saved| saved.window = Some(place));
    }
}

pub fn store_material(material: Material, cx: &mut App) {
    theme::choose_material(material, cx);
    persist::update(cx, |saved| saved.material = Some(material));
    let background = background(cx);
    for handle in cx.windows() {
        handle
            .update(cx, |_, window, _| {
                window.set_background_appearance(background);
            })
            .ok();
    }
}
