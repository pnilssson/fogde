use super::FolderBrowser;
use crate::assets::Glyph;
use crate::theme::Type;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Disableable, Icon, Sizable, WindowExt, h_flex, v_flex};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

impl FolderBrowser {
    fn rows(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let muted = cx.theme().muted_foreground;
        let hover = crate::theme::tokens(cx).hover;
        let mut rows = v_flex();
        for folder in self
            .folders
            .as_ref()
            .map(|folders| folders.folders.clone())
            .unwrap_or_default()
        {
            let (open, remove, name) = (
                folder.path.clone(),
                folder.path.clone(),
                folder.name.clone(),
            );
            rows = rows.child(
                h_flex()
                    .id(SharedString::from(folder.path.clone()))
                    .group("folder")
                    .gap_2()
                    .px_2()
                    .h_8()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|row| row.bg(hover))
                    .child(Icon::new(Glyph::Folder).small())
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .type_body()
                            .child(folder.name.clone()),
                    )
                    .child(
                        Button::new(SharedString::from(format!("delete-{}", folder.path)))
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(Glyph::Trash))
                            .tooltip(format!("Delete {}", folder.name))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.remove(remove.clone(), name.clone(), window, cx);
                            })),
                    )
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.open(open.clone(), window, cx)),
                    ),
            );
        }
        if self
            .folders
            .as_ref()
            .is_some_and(|folders| folders.folders.is_empty())
        {
            rows = rows.child(
                div()
                    .p_2()
                    .type_body()
                    .text_color(muted)
                    .child("No folders here"),
            );
        }
        rows
    }
}

impl Render for FolderBrowser {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let parent = self
            .folders
            .as_ref()
            .and_then(|folders| folders.parent.clone());
        let current = self.folders.as_ref().map(|folders| folders.path.clone());
        let loading = self.loading.is_some();
        let rows = self.rows(cx);

        v_flex()
            .gap_2()
            .min_w_0()
            .child(
                Input::new(&self.path)
                    .prefix(
                        Button::new("up")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(Glyph::ArrowUp))
                            .tooltip("Up one folder")
                            .disabled(parent.is_none())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if let Some(parent) = parent.clone() {
                                    this.open(parent, window, cx);
                                }
                            })),
                    )
                    .when(loading, |input| input.suffix(Spinner::new().small())),
            )
            .when_some(self.error.clone(), |browser, error| {
                browser.child(div().type_body().child(error))
            })
            .when(self.naming, |browser| {
                browser.child(
                    v_flex()
                        .gap_1()
                        .child(
                            Input::new(&self.name).suffix(
                                Button::new("create-folder")
                                    .ghost()
                                    .xsmall()
                                    .label("Create")
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.create(window, cx)),
                                    ),
                            ),
                        )
                        .when(self.invalid, |field| {
                            field.child(
                                div()
                                    .type_caption()
                                    .text_color(cx.theme().danger)
                                    .child("Enter a name for the folder"),
                            )
                        }),
                )
            })
            .child(
                div()
                    .id("folders")
                    .h(px(256.))
                    .overflow_y_scroll()
                    .child(rows),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("new-folder")
                            .ghost()
                            .small()
                            .label("New folder")
                            .disabled(current.is_none() || self.naming)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.naming = true;
                                this.name.update(cx, |name, cx| name.focus(window, cx));
                                cx.notify();
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("cancel")
                            .ghost()
                            .small()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("choose")
                            .secondary()
                            .small()
                            .loading(self.busy)
                            .disabled(current.is_none() || self.busy)
                            .label(self.action.clone())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if let Some(path) = current.clone() {
                                    (this.on_choose)(path, cx.entity().downgrade(), window, cx);
                                }
                            })),
                    ),
            )
    }
}
