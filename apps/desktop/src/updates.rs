mod deb;

use std::sync::Arc;
use std::time::Duration;

use cargo_packager_updater::semver::Version;
use cargo_packager_updater::{Config, Update, UpdaterBuilder, url::Url};
use gpui_kit::component::Icon;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{
    AnyElement, App, AppContext, Context, Entity, Global, IntoElement, ParentElement, Styled, Task,
    div,
};

use crate::assets::Glyph;
use crate::notice;
use crate::runtime;

const HOUR: Duration = Duration::from_hours(1);
const FEED: &str = "https://github.com/pnilssson/fogde/releases/latest/download/latest.json";
const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDI5MjZBNEYyMkIwRjZDNUUKUldSZWJBOHI4cVFtS1kwdjFETkp2NnF0dnZvajFiZTMvMmFhbHJVRk9mcHJvRXhxQms4NXhGalMK";

#[derive(Clone, Copy)]
enum Install {
    Bundle,
    Deb,
}

struct Ready {
    update: Arc<Update>,
    bytes: Arc<Vec<u8>>,
    kind: Install,
}

pub struct Updates {
    ready: Option<Ready>,
    installing: bool,
    warned: bool,
    _checking: Task<()>,
}

struct Hub(Entity<Updates>);

impl Global for Hub {}

fn newer(current: &Version, offered: &Version) -> bool {
    offered > current
}

fn kind() -> Option<Install> {
    if !cfg!(target_os = "linux") || std::env::var_os("APPIMAGE").is_some() {
        return Some(Install::Bundle);
    }

    deb::installed().then_some(Install::Deb)
}

fn check(ready: Option<&str>) -> Result<Option<(Install, Update, Vec<u8>)>, String> {
    let Some(kind) = kind() else {
        return Ok(None);
    };
    let current = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|error| error.to_string())?;
    let endpoint = Url::parse(FEED).map_err(|error| error.to_string())?;
    let config = Config {
        endpoints: vec![endpoint],
        pubkey: KEY.into(),
        windows: None,
    };
    let mut builder = UpdaterBuilder::new(current, config)
        .version_comparator(|current, release| newer(&current, &release.version));
    if let Install::Deb = kind {
        builder = builder.target(format!("linux-{}-deb", std::env::consts::ARCH));
    }
    let updater = builder.build().map_err(|error| error.to_string())?;
    let Some(update) = updater.check().map_err(|error| error.to_string())? else {
        return Ok(None);
    };
    if ready == Some(update.version.as_str()) {
        return Ok(None);
    }

    let bytes = update.download().map_err(|error| error.to_string())?;
    Ok(Some((kind, update, bytes)))
}

impl Updates {
    fn new(cx: &mut Context<Self>) -> Self {
        let checking = cx.spawn(async move |this, cx| {
            if cfg!(debug_assertions) {
                return;
            }
            loop {
                let ready = this
                    .read_with(cx, |this, _| {
                        this.ready
                            .as_ref()
                            .map(|ready| ready.update.version.clone())
                    })
                    .ok()
                    .flatten();
                let task = cx.update(|cx| {
                    runtime::spawn(cx, async move {
                        tokio::task::spawn_blocking(move || check(ready.as_deref())).await
                    })
                });
                let result = task
                    .await
                    .and_then(Result::ok)
                    .unwrap_or_else(|| Err("The update check stopped.".into()));
                let stop = this
                    .update(cx, |this, cx| {
                        match result {
                            Ok(Some((kind, update, bytes))) => {
                                this.ready = Some(Ready {
                                    update: Arc::new(update),
                                    bytes: Arc::new(bytes),
                                    kind,
                                });
                            }
                            Err(error) if !this.warned => {
                                this.warned = true;
                                notice::error("Could not check for updates", &error, cx);
                            }
                            Ok(None) | Err(_) => {}
                        }
                        cx.notify();
                    })
                    .is_err();
                if stop {
                    break;
                }
                cx.background_executor().timer(HOUR).await;
            }
        });
        Self {
            ready: None,
            installing: false,
            warned: false,
            _checking: checking,
        }
    }

    fn install(&mut self, cx: &mut Context<Self>) {
        let Some(ready) = &self.ready else {
            return;
        };
        self.installing = true;
        cx.notify();
        let (update, bytes, kind) = (ready.update.clone(), ready.bytes.clone(), ready.kind);
        let task = runtime::spawn(cx, async move {
            tokio::task::spawn_blocking(move || match kind {
                Install::Bundle => update.install((*bytes).clone()),
                Install::Deb => deb::install(&bytes).map_err(cargo_packager_updater::Error::Io),
            })
            .await
        });
        cx.spawn(async move |this, cx| {
            let result = task.await.and_then(Result::ok).unwrap_or_else(|| {
                Err(cargo_packager_updater::Error::Io(std::io::Error::other(
                    "The update stopped.",
                )))
            });
            this.update(cx, |this, cx| {
                this.installing = false;
                match result {
                    Ok(()) => cx.restart(),
                    Err(error) => notice::error("Could not update Fogde", &error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

pub fn init(cx: &mut App) {
    let updates = cx.new(Updates::new);
    cx.set_global(Hub(updates));
}

pub fn entity(cx: &App) -> Option<Entity<Updates>> {
    cx.try_global::<Hub>().map(|hub| hub.0.clone())
}

pub fn button(cx: &App) -> Option<AnyElement> {
    let updates = cx.try_global::<Hub>()?.0.clone();
    let state = updates.read(cx);
    state.ready.as_ref()?;
    let installing = state.installing;
    Some(
        div()
            .p_2()
            .child(
                Button::new("update")
                    .secondary()
                    .w_full()
                    .justify_start()
                    .loading(installing)
                    .icon(Icon::new(Glyph::ArrowCircleUp))
                    .label("Update ready")
                    .on_click(move |_, _, cx| {
                        updates.update(cx, Updates::install);
                    }),
            )
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_a_feed_version_that_is_not_newer() {
        let current = Version::new(0, 1, 40);
        assert!(newer(&current, &Version::new(0, 1, 41)));
        assert!(!newer(&current, &Version::new(0, 1, 40)));
        assert!(!newer(&current, &Version::new(0, 1, 39)));
    }
}
