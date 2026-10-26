use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use protocol::stats::Stats;
use protocol::terminals::{Changes, Status};

use super::desk::split::Blank;
use super::status_dot;
use super::tiles::tile::Dragged;
use crate::assets::Glyph;
use crate::state::settings;
use crate::theme::{self, Type};

type OnClose = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct TileBar {
    id: ElementId,
    dragged: Option<Dragged>,
    active: bool,
    icon: AnyElement,
    title: SharedString,
    status: Option<Status>,
    place: Option<(String, String)>,
    branch: Option<String>,
    changes: Option<Changes>,
    on_close: Option<OnClose>,
}

impl TileBar {
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl IntoElement,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            dragged: None,
            active: false,
            icon: icon.into_any_element(),
            title: title.into(),
            status: None,
            place: None,
            branch: None,
            changes: None,
            on_close: None,
        }
    }

    pub fn dragged(mut self, dragged: Option<Dragged>) -> Self {
        self.dragged = dragged;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn place(mut self, place: Option<(String, String)>) -> Self {
        self.place = place;
        self
    }

    pub fn branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self
    }

    pub fn changes(mut self, changes: Option<Changes>) -> Self {
        self.changes = changes;
        self
    }

    pub fn on_close(mut self, on_close: Option<OnClose>) -> Self {
        self.on_close = on_close;
        self
    }
}

struct Part {
    counts: Vec<(String, Option<gpui_kit::Hsla>)>,
    sentence: String,
}

fn counted(number: u32, one: &str, many: &str) -> String {
    format!("{number} {}", if number == 1 { one } else { many })
}

fn parts(changes: &Changes, stats: Stats, cx: &App) -> Vec<Part> {
    let tokens = theme::tokens(cx);
    let mut parts = Vec::new();

    if stats.behind
        && let Some(base) = changes.base.as_ref().filter(|base| base.behind > 0)
    {
        parts.push(Part {
            counts: vec![(format!("↓{} {}", base.behind, base.name), None)],
            sentence: format!(
                "{} behind {}",
                counted(base.behind, "commit", "commits"),
                base.name
            ),
        });
    }

    if stats.lines && (changes.added > 0 || changes.removed > 0) {
        let since = changes.base.as_ref().map_or_else(
            || "since the last commit".to_owned(),
            |base| format!("since {}", base.name),
        );
        let mut counts = Vec::new();
        if changes.added > 0 {
            counts.push((format!("+{}", changes.added), Some(tokens.success)));
        }
        if changes.removed > 0 {
            counts.push((format!("−{}", changes.removed), Some(tokens.destructive)));
        }
        parts.push(Part {
            counts,
            sentence: format!(
                "{} added and {} removed {since}",
                counted(changes.added, "line", "lines"),
                changes.removed
            ),
        });
    }

    if stats.upstream
        && let Some(upstream) = changes
            .upstream
            .filter(|upstream| upstream.ahead > 0 || upstream.behind > 0)
    {
        let mut counts = Vec::new();
        if upstream.ahead > 0 {
            counts.push((format!("↑{}", upstream.ahead), None));
        }
        if upstream.behind > 0 {
            counts.push((format!("↓{}", upstream.behind), None));
        }
        parts.push(Part {
            counts,
            sentence: format!(
                "{} to push and {} to pull",
                counted(upstream.ahead, "commit", "commits"),
                upstream.behind
            ),
        });
    }

    parts
}

fn stats(id: &ElementId, changes: Option<&Changes>, cx: &App) -> Option<AnyElement> {
    let parts = parts(changes?, settings::stats(cx)?, cx);
    if parts.is_empty() {
        return None;
    }
    let sentences: Vec<String> = parts.iter().map(|part| part.sentence.clone()).collect();

    Some(
        h_flex()
            .id(ElementId::Name(format!("{id:?}-stats").into()))
            .flex_shrink_0()
            .gap_2()
            .children(parts.into_iter().map(|part| {
                h_flex().gap_1().children(
                    part.counts
                        .into_iter()
                        .map(|(text, tone)| div().when_some(tone, Styled::text_color).child(text)),
                )
            }))
            .tooltip(move |window, cx| {
                let lines = sentences.clone();
                Tooltip::element(move |_, _| {
                    v_flex().children(lines.iter().map(|line| div().child(line.clone())))
                })
                .build(window, cx)
            })
            .into_any_element(),
    )
}

impl RenderOnce for TileBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let title = self.title.clone();
        let stats = stats(&self.id, self.changes.as_ref(), cx);

        h_flex()
            .id(self.id.clone())
            .h_8()
            .flex_shrink_0()
            .gap_2()
            .px_2()
            .border_b_1()
            .border_color(theme::tokens(cx).border)
            .type_caption()
            .font_family(theme::MONO)
            .when(!self.active, |bar| bar.text_color(muted))
            .when_some(self.dragged, |bar, dragged| {
                bar.cursor_grab().on_drag(dragged, |_, _, _, cx| {
                    gpui_kit::AppContext::new(cx, |_| Blank)
                })
            })
            .child(self.icon)
            .child(
                h_flex()
                    .min_w_0()
                    .flex_1()
                    .gap_2()
                    .child(div().truncate().child(self.title))
                    .when_some(self.status, |row, status| {
                        row.children(status_dot(status, cx))
                    }),
            )
            .when_some(self.place, |bar, (label, path)| {
                bar.child(
                    div()
                        .id("place")
                        .min_w_0()
                        .max_w(gpui_kit::relative(0.33))
                        .truncate()
                        .text_color(muted)
                        .child(label)
                        .tooltip(move |window, cx| Tooltip::new(path.clone()).build(window, cx)),
                )
            })
            .when_some(self.branch, |bar, branch| {
                bar.child(
                    h_flex()
                        .min_w_0()
                        .max_w(gpui_kit::relative(0.33))
                        .gap_2()
                        .child(Icon::new(Glyph::GitBranch).small())
                        .child(div().truncate().child(branch)),
                )
            })
            .children(stats)
            .when_some(self.on_close, |bar, on_close| {
                bar.child(
                    Button::new("close")
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(Glyph::X))
                        .tooltip(format!("Close {title}"))
                        .on_click(on_close),
                )
            })
            .min_h(px(32.))
    }
}
