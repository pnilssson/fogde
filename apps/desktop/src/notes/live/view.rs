use gpui_kit::accesskit::ActionData;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, v_flex};
use gpui_kit::{
    AccessibleAction, AnyElement, App, Context, FontStyle, FontWeight, HighlightStyle,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Render, Role, SharedString,
    StatefulInteractiveElement, StrikethroughStyle, Styled, StyledText, TextLayout, Toggled,
    UnderlineStyle, Window, combine_highlights, div, prelude::FluentBuilder, px,
};

use super::parse::{Kind, Mark};
use super::preview::{self, Line, Widget};
use super::{CONTEXT, Live, Shown, keys};
use crate::assets::Glyph;
use crate::num;
use crate::theme::{self, Type};

fn style(mark: &Mark, cx: &App) -> HighlightStyle {
    let link = cx.theme().link;
    match mark {
        Mark::Strong => HighlightStyle {
            font_weight: Some(FontWeight::BOLD),
            ..HighlightStyle::default()
        },
        Mark::Emphasis => HighlightStyle {
            font_style: Some(FontStyle::Italic),
            ..HighlightStyle::default()
        },
        Mark::Strike => HighlightStyle {
            strikethrough: Some(StrikethroughStyle {
                thickness: px(1.),
                color: None,
            }),
            ..HighlightStyle::default()
        },
        Mark::Code => HighlightStyle {
            background_color: Some(cx.theme().muted),
            ..HighlightStyle::default()
        },
        Mark::Muted => HighlightStyle {
            color: Some(cx.theme().muted_foreground),
            ..HighlightStyle::default()
        },
        Mark::Link(_) | Mark::Wikilink(_) => HighlightStyle {
            color: Some(link),
            underline: Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(link),
                wavy: false,
            }),
            ..HighlightStyle::default()
        },
    }
}

fn checkbox(at: usize, checked: bool, cx: &mut Context<Live>) -> AnyElement {
    let theme = cx.theme();
    let (border, fill, mark) = (theme.border, theme.primary, theme.primary_foreground);
    div()
        .id(SharedString::from(format!("task-{at}")))
        .flex_none()
        .mt(px(2.))
        .mr_2()
        .size(px(14.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .border_1()
        .border_color(if checked { fill } else { border })
        .cursor_pointer()
        .when(checked, |box_| {
            box_.bg(fill)
                .child(Icon::new(Glyph::Check).xsmall().text_color(mark))
        })
        .role(Role::CheckBox)
        .aria_toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        })
        .aria_label(if checked { "Done" } else { "Not done" })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
                this.toggle(at, cx);
            }),
        )
        .on_a11y_action(AccessibleAction::Click, {
            let entity = cx.entity();
            move |_, _, cx| entity.update(cx, |this, cx| this.toggle(at, cx))
        })
        .into_any_element()
}

impl Live {
    fn row(line: &Line, cx: &mut Context<Self>) -> (AnyElement, TextLayout) {
        let highlights: Vec<_> = combine_highlights(
            line.marks
                .iter()
                .map(|(range, mark)| (range.clone(), style(mark, cx))),
            [],
        )
        .collect();
        let code: Vec<_> = line
            .marks
            .iter()
            .filter(|(_, mark)| *mark == Mark::Code)
            .map(|(range, _)| (range.clone(), SharedString::from(theme::MONO)))
            .collect();
        let text = StyledText::new(line.text.clone())
            .with_highlights(highlights)
            .with_font_family_overrides(code);
        let layout = text.layout().clone();
        let content = div().flex_1().min_w_0().child(text);
        let muted = cx.theme().muted_foreground;
        let indent = px(4. * num::real_usize(line.indent));

        let row = div()
            .w_full()
            .flex()
            .items_start()
            .map(|row| match line.kind {
                Kind::Heading(1) => row.type_display(),
                Kind::Heading(2 | 3) => row.type_title(),
                Kind::Heading(_) => row.font_weight(FontWeight::SEMIBOLD),
                Kind::Code => row.type_code().bg(cx.theme().muted).px_2(),
                Kind::Table => row.type_code(),
                Kind::Quote => row
                    .border_l_2()
                    .border_color(cx.theme().border)
                    .pl_4()
                    .text_color(muted),
                Kind::Body => row,
            });
        let row = match &line.widget {
            Widget::None => row.child(content),
            Widget::Bullet => row
                .pl(indent)
                .child(div().flex_none().w_4().text_color(muted).child("•"))
                .child(content),
            Widget::Task { checked, at } => row
                .pl(indent)
                .child(checkbox(*at, *checked, cx))
                .child(content),
            Widget::Rule => row.relative().child(content).child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top_1_2()
                    .h(px(1.))
                    .bg(cx.theme().border),
            ),
        };
        (row.into_any_element(), layout)
    }
}

impl Render for Live {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active(window);
        let lines = preview::lines(&self.doc.text, &self.parsed, &active);
        let mut shown = Vec::with_capacity(lines.len());
        let mut rows = Vec::with_capacity(lines.len());
        for line in lines {
            let (row, layout) = Self::row(&line, cx);
            rows.push(row);
            shown.push(Shown { line, layout });
        }
        *self.shown.borrow_mut() = shown;

        let focused = self.focus.is_focused(window);
        let root = div()
            .id("note")
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .role(Role::MultilineTextInput)
            .aria_label("Note")
            .when(window.is_a11y_active(), |root| {
                root.aria_value(self.doc.text.clone())
            })
            .on_a11y_action(AccessibleAction::Focus, {
                let focus = self.focus.clone();
                move |_, window, cx| focus.focus(window, cx)
            })
            .on_a11y_action(AccessibleAction::SetValue, {
                let entity = cx.entity();
                move |data, _, cx| {
                    if let Some(ActionData::Value(value)) = data {
                        let value = value.replace("\r\n", "\n");
                        entity.update(cx, |this, cx| this.set(&value, cx));
                    }
                }
            })
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .cursor_text()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_move(cx.listener(|this, event, _, cx| this.mouse_move(event, cx)))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.mouse_up()),
            )
            .child(
                div().w_full().flex().justify_center().child(
                    v_flex()
                        .relative()
                        .w_full()
                        .max_w(px(768.))
                        .px_8()
                        .py_6()
                        .type_body()
                        .child(self.background(focused, cx))
                        .children(rows)
                        .child(self.foreground(focused, cx)),
                ),
            );
        keys::listen(root, cx)
    }
}
