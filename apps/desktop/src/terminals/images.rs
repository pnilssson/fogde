use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use gpui_kit::{AppContext, ClipboardEntry, ClipboardItem, Context, ImageFormat};
use protocol::terminals::{IMAGE_LIMIT, Image};

use super::outgoing::Outgoing;
use super::view::TerminalView;
use crate::notice;

enum Pasted {
    Image(Image, Vec<u8>),
    File(PathBuf),
}

fn format(format: ImageFormat) -> Option<Image> {
    match format {
        ImageFormat::Png => Some(Image::Png),
        ImageFormat::Jpeg => Some(Image::Jpeg),
        ImageFormat::Gif => Some(Image::Gif),
        ImageFormat::Webp => Some(Image::Webp),
        _ => None,
    }
}

pub fn kind(path: &Path) -> Option<Image> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some(Image::Png),
        "jpg" | "jpeg" => Some(Image::Jpeg),
        "gif" => Some(Image::Gif),
        "webp" => Some(Image::Webp),
        _ => None,
    }
}

fn name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

pub fn attach(view: &mut TerminalView, item: ClipboardItem, cx: &mut Context<TerminalView>) {
    let pasted = item
        .entries
        .into_iter()
        .flat_map(|entry| match entry {
            ClipboardEntry::Image(image) => format(image.format)
                .map(|kind| Pasted::Image(kind, image.bytes))
                .into_iter()
                .collect::<Vec<_>>(),
            ClipboardEntry::ExternalPaths(paths) => {
                paths.paths().iter().cloned().map(Pasted::File).collect()
            }
            ClipboardEntry::String(_) => Vec::new(),
        })
        .collect();
    send(view, pasted, cx);
}

pub fn drop_paths(view: &mut TerminalView, paths: &[PathBuf], cx: &mut Context<TerminalView>) {
    send(view, paths.iter().cloned().map(Pasted::File).collect(), cx);
}

fn send(view: &mut TerminalView, pasted: Vec<Pasted>, cx: &mut Context<TerminalView>) {
    if pasted.is_empty() {
        return;
    }
    if !view.watching() {
        notice::error("Could not paste", &"The terminal is not connected yet.", cx);
        return;
    }

    cx.spawn(async move |this, cx| {
        for item in pasted {
            let loaded = match item {
                Pasted::Image(kind, bytes) => Ok((kind, bytes)),
                Pasted::File(path) => match kind(&path) {
                    None => Err(format!(
                        "{} is not a PNG, JPEG, GIF or WebP image.",
                        name(&path)
                    )),
                    Some(kind) => cx
                        .background_spawn(async move { std::fs::read(&path) })
                        .await
                        .map(|bytes| (kind, bytes))
                        .map_err(|error| error.to_string()),
                },
            };
            let checked = loaded.and_then(|(kind, bytes)| {
                if bytes.len() > IMAGE_LIMIT {
                    return Err(format!(
                        "The image is larger than {} MB.",
                        IMAGE_LIMIT / 1024 / 1024
                    ));
                }
                Ok((kind, STANDARD.encode(bytes)))
            });

            let sent = this.update(cx, |view, cx| match checked {
                Ok((kind, data)) => view.send(Outgoing::Image(kind, data)),
                Err(error) => notice::error("Could not paste", &error, cx),
            });
            if sent.is_err() {
                break;
            }
        }
    })
    .detach();
}
