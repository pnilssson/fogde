use super::NotesTile;
use crate::assets::Glyph;
use crate::state::store;
use crate::theme::Type;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder,
};

impl NotesTile {
    fn message(title: &'static str, description: String, cx: &App) -> AnyElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .child(div().type_title().child(title))
            .child(
                div()
                    .type_body()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            )
            .into_any_element()
    }

    fn content(&self, cx: &mut Context<Self>) -> AnyElement {
        let list = store(cx).notes.read(cx).list(&self.machine).cloned();
        if let Some((_, editor)) = &self.editor {
            return editor.clone().into_any_element();
        }
        let Some(list) = list else {
            return div().into_any_element();
        };
        match &self.note {
            Some(path) if list.notes.iter().any(|note| note.path == *path) => {
                if self.failed {
                    h_flex()
                        .gap_2()
                        .p_2()
                        .child(div().type_caption().child("Could not read the note"))
                        .child(
                            Button::new("retry-note")
                                .secondary()
                                .small()
                                .label("Try again")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.failed = false;
                                    this.sync(window, cx);
                                })),
                        )
                        .into_any_element()
                } else {
                    h_flex()
                        .gap_2()
                        .p_2()
                        .child(Spinner::new().small())
                        .child(div().type_caption().child("Opening note…"))
                        .into_any_element()
                }
            }
            Some(_) => Self::message(
                "This note was moved or deleted",
                "Pick another note from the list.".into(),
                cx,
            ),
            None => {
                let description = if list.notes.is_empty() {
                    format!("Notes you add to {} show here.", list.root)
                } else {
                    "Pick a note from the list.".into()
                };
                Self::message("No note open", description, cx)
            }
        }
    }
}

impl Render for NotesTile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entry = store(cx).notes.read(cx).entry(&self.machine);
        let error = entry
            .and_then(|entry| entry.error.clone())
            .filter(|_| entry.and_then(|entry| entry.list.as_ref()).is_none());
        let loading = entry.and_then(|entry| entry.list.as_ref()).is_none();
        let border = cx.theme().border;

        let body = if error.is_some() {
            h_flex()
                .gap_2()
                .p_2()
                .child(
                    div()
                        .type_caption()
                        .child("Could not open the notes folder"),
                )
                .child(
                    Button::new("retry-notes")
                        .secondary()
                        .small()
                        .label("Try again")
                        .on_click(cx.listener(|this, _, _, cx| {
                            let machine = this.machine.clone();
                            store(cx)
                                .notes
                                .clone()
                                .update(cx, |notes, cx| notes.reload(&machine, cx));
                        })),
                )
                .into_any_element()
        } else if loading {
            h_flex()
                .gap_2()
                .p_2()
                .child(Spinner::new().small())
                .child(div().type_caption().child("Opening notes…"))
                .into_any_element()
        } else {
            h_flex()
                .size_full()
                .min_h_0()
                .child(if self.browsing {
                    self.pane(cx)
                } else {
                    div()
                        .flex_shrink_0()
                        .p_1()
                        .border_r_1()
                        .border_color(border)
                        .child(
                            Button::new("show-notes")
                                .ghost()
                                .small()
                                .icon(Icon::new(Glyph::SidebarSimple))
                                .tooltip("Show notes list")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.browsing = true;
                                    cx.notify();
                                })),
                        )
                        .into_any_element()
                })
                .child(div().flex_1().min_w_0().h_full().child(self.content(cx)))
                .into_any_element()
        };

        div()
            .id("notes")
            .track_focus(&self.focus)
            .size_full()
            .when(true, |tile| tile.child(body))
    }
}
