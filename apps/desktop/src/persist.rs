mod writer;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{App, AppContext, Global, Task};
use protocol::theme::Material;
use serde::{Deserialize, Serialize};
use writer::Writer;

const SETTLE: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Place {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Saved {
    pub material: Option<Material>,
    pub window: Option<Place>,
    pub workspace: Option<(String, String)>,
    pub checkout: Option<String>,
    pub layouts: BTreeMap<String, serde_json::Value>,
    pub closed: BTreeSet<String>,
    pub sort: Option<String>,
    pub recent: BTreeMap<String, Vec<String>>,
}

struct Persist {
    writer: Option<Arc<Writer>>,
    number: u64,
    saved: Saved,
    writing: Option<Task<()>>,
}

impl Global for Persist {}

fn path() -> Option<PathBuf> {
    host::fogde()
        .ok()
        .map(|fogde| fogde.join(".app").join("desktop.json"))
}

pub fn load() -> Saved {
    path()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| {
            serde_json::from_slice(&bytes)
                .inspect_err(|error| tracing::warn!(%error, "could not read the saved app state"))
                .ok()
        })
        .unwrap_or_default()
}

pub fn init(saved: Saved, cx: &mut App) {
    cx.set_global(Persist {
        writer: path().map(|path| Arc::new(Writer::new(path))),
        number: 0,
        saved,
        writing: None,
    });
    cx.on_app_quit(|cx| {
        flush(cx);
        async {}
    })
    .detach();
}

pub fn saved(cx: &App) -> &Saved {
    &cx.global::<Persist>().saved
}

pub fn update(cx: &mut App, change: impl FnOnce(&mut Saved)) {
    let persist = cx.global_mut::<Persist>();
    change(&mut persist.saved);
    let Some(writer) = persist.writer.clone() else {
        return;
    };
    persist.number += 1;
    let (number, snapshot) = (persist.number, persist.saved.clone());

    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(SETTLE).await;
        cx.background_spawn(async move { write(&writer, number, &snapshot) })
            .await;
    });
    cx.global_mut::<Persist>().writing = Some(task);
}

fn flush(cx: &mut App) {
    let persist = cx.global_mut::<Persist>();
    if persist.writing.take().is_some()
        && let Some(writer) = &persist.writer
    {
        write(writer, persist.number, &persist.saved);
    }
}

fn write(writer: &Writer, number: u64, saved: &Saved) {
    if let Err(error) = writer.write(number, saved) {
        tracing::warn!(%error, "could not save the app state");
    }
}
