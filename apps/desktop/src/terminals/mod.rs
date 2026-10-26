mod access;
mod boxes;
mod colors;
mod element;
mod images;
mod input;
mod keys;
mod layout;
mod links;
mod outgoing;
mod route;
mod scroll;
mod scrollbar;
mod select;
mod view;
mod watch;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex};
use gpui_kit::{
    Context, ExternalPaths, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    actions, div, prelude::FluentBuilder, px,
};

pub use view::{Status, TerminalView};

use crate::assets::Glyph;
use crate::theme::{self, Type};
use element::TerminalElement;

actions!(terminal, [Copy, Paste]);

pub const CONTEXT: &str = "Terminal";

impl TerminalView {
    fn status_bar(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if !self.connected {
            return None;
        }
        let notice = match self.status {
            Status::Ready => return None,
            Status::Opening => "Opening terminal…",
            Status::Ended => "The terminal ended",
            Status::Failed => "Could not show the terminal",
        };

        Some(
            h_flex()
                .gap_2()
                .p_2()
                .when(self.status == Status::Opening, |row| {
                    row.child(Spinner::new().small())
                })
                .child(div().type_caption().child(notice))
                .when(self.status != Status::Opening, |row| {
                    row.child(
                        Button::new("reopen")
                            .small()
                            .secondary()
                            .label("Reopen terminal")
                            .on_click(cx.listener(|this, _, _, cx| this.retry(cx))),
                    )
                }),
        )
    }

    fn copied_notice(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        self.copied.as_ref()?;
        let tokens = theme::tokens(cx);

        Some(
            div()
                .absolute()
                .bottom_4()
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    h_flex()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .rounded(cx.theme().radius_lg)
                        .bg(cx.theme().popover)
                        .text_color(cx.theme().popover_foreground)
                        .shadow(tokens.shadows[4].clone())
                        .type_body()
                        .child(Icon::new(Glyph::Check).small())
                        .child("Copied"),
                ),
        )
    }

    fn preedit(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let marked = self.marked.clone()?;
        let cell = self.cell?;
        let cursor = self.screen.term().grid().cursor.point;
        let column = u16::try_from(cursor.column.0).ok()?;
        let row = u16::try_from(cursor.line.0).ok()?;

        Some(
            div()
                .absolute()
                .left(px(8.) + cell.width * f32::from(column))
                .top(px(8.) + cell.height * f32::from(row))
                .bg(theme::tokens(cx).surface[0])
                .border_b_1()
                .border_color(cx.theme().foreground)
                .type_code()
                .child(marked),
        )
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.place == view::Place::Claiming {
            self.place = view::Place::Active;
            self.focus(window, cx);
        }
        self.palette = colors::Palette::new(theme::tokens(cx));
        let accessible =
            window.is_a11y_active() || crate::state::store(cx).settings.read(cx).screen_reader();
        let access = accessible.then(|| {
            let label = crate::state::store(cx)
                .workspaces
                .read(cx)
                .terminal(&self.machine, &self.id)
                .map_or_else(
                    || "Terminal".to_owned(),
                    |(_, terminal)| terminal.name.clone(),
                );
            access::snapshot(&self.screen, label)
        });

        div()
            .id("terminal")
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .track_focus(&self.focus)
            .key_context(CONTEXT)
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(|this, _: &Copy, _, cx| {
                select::copy(this, cx);
                select::clear(this);
            }))
            .on_action(cx.listener(|this, _: &Paste, _, cx| select::paste(this, cx)))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                images::drop_paths(this, paths.paths(), cx);
            }))
            .children(self.status_bar(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .p_2()
                    .child(TerminalElement::new(cx.entity(), access))
                    .children(self.scrollbar(cx))
                    .children(self.copied_notice(cx))
                    .children(self.preedit(cx)),
            )
            .min_w(px(0.))
    }
}
