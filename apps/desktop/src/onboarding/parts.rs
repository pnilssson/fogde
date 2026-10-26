use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::assets::Glyph;
use crate::theme::{self, Type};

pub type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

pub fn section(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Div {
    v_flex()
        .max_w(px(384.))
        .items_start()
        .gap_4()
        .child(div().type_display().child(title.into()))
        .child(div().type_body().child(body.into()))
}

pub fn bars(total: usize, position: usize, cx: &App) -> AnyElement {
    let border = cx.theme().border;
    let fill = cx.theme().foreground;
    h_flex()
        .id("steps")
        .role(gpui_kit::Role::ProgressIndicator)
        .aria_label(format!("Step {} of {total}", position + 1))
        .gap_2()
        .children((0..total).map(|index| {
            let share = match index.cmp(&position) {
                std::cmp::Ordering::Less => 1.,
                std::cmp::Ordering::Equal => 0.25,
                std::cmp::Ordering::Greater => 0.,
            };
            div()
                .h_1()
                .w_6()
                .rounded_full()
                .bg(border)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(fill)
                        .w(gpui_kit::relative(share)),
                )
        }))
        .into_any_element()
}

pub fn done(label: &'static str, cx: &App) -> AnyElement {
    h_flex()
        .gap_2()
        .px_2()
        .h_6()
        .rounded_md()
        .bg(cx.theme().secondary)
        .type_caption()
        .child(div().size_2().rounded_full().bg(theme::tokens(cx).success))
        .child(label)
        .into_any_element()
}

pub struct Connection {
    pub id: ElementId,
    pub name: SharedString,
    pub icon: Option<AnyElement>,
    pub action: &'static str,
    pub pending: &'static str,
    pub connected: bool,
    pub busy: bool,
    pub on_connect: Handler,
    pub on_cancel: Option<Handler>,
}

pub fn connection(row: Connection, cx: &App) -> AnyElement {
    let Connection {
        id,
        name,
        icon,
        action,
        pending,
        connected,
        busy,
        on_connect,
        on_cancel,
    } = row;
    let status = if connected {
        done("Connected", cx)
    } else {
        h_flex()
            .gap_2()
            .when_some(on_cancel.filter(|_| busy), |row, cancel| {
                row.child(
                    Button::new("cancel")
                        .ghost()
                        .small()
                        .label("Cancel")
                        .on_click(move |event, window, cx| cancel(event, window, cx)),
                )
            })
            .child(
                Button::new(id)
                    .secondary()
                    .small()
                    .loading(busy)
                    .label(if busy { pending } else { action })
                    .on_click(move |event, window, cx| {
                        if !busy {
                            on_connect(event, window, cx);
                        }
                    }),
            )
            .into_any_element()
    };

    h_flex()
        .h_9()
        .w_full()
        .justify_between()
        .child(h_flex().gap_2().type_body().children(icon).child(name))
        .child(status)
        .into_any_element()
}

pub fn nav(
    back: Option<Handler>,
    label: &'static str,
    busy: bool,
    disabled: bool,
    next: Handler,
) -> AnyElement {
    h_flex()
        .gap_2()
        .when_some(back, |row, back| {
            row.child(
                Button::new("back")
                    .ghost()
                    .small()
                    .label("Back")
                    .on_click(move |event, window, cx| back(event, window, cx)),
            )
        })
        .child(
            Button::new("continue")
                .secondary()
                .small()
                .loading(busy)
                .disabled(disabled || busy)
                .label(label)
                .on_click(move |event, window, cx| next(event, window, cx)),
        )
        .into_any_element()
}

pub struct Code {
    pub value: String,
    pub url: String,
    pub site: SharedString,
    pub hint: Option<String>,
    pub copied: bool,
    pub on_copy: Handler,
    pub on_cancel: Option<Handler>,
}

pub fn device_code(code: Code, cx: &App) -> AnyElement {
    let url = code.url.clone();
    let muted = cx.theme().muted_foreground;
    v_flex()
        .items_center()
        .gap_8()
        .child(
            h_flex()
                .gap_1()
                .type_display()
                .child("Enter this code on")
                .child(
                    div()
                        .id("site")
                        .underline()
                        .cursor_pointer()
                        .child(code.site.clone())
                        .on_click(move |_, _, cx| cx.open_url(&url)),
                )
                .when_some(code.hint.clone(), |row, hint| {
                    row.child(
                        Button::new("hint")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(Glyph::Info))
                            .tooltip(hint),
                    )
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .type_display()
                        .font_family(theme::MONO)
                        .child(code.value.clone()),
                )
                .child({
                    let on_copy = code.on_copy.clone();
                    Button::new("copy")
                        .ghost()
                        .small()
                        .icon(Icon::new(if code.copied {
                            Glyph::Check
                        } else {
                            Glyph::Copy
                        }))
                        .tooltip(if code.copied { "Copied" } else { "Copy code" })
                        .on_click(move |event, window, cx| on_copy(event, window, cx))
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .type_body()
                .text_color(muted)
                .child(gpui_kit::component::spinner::Spinner::new().small())
                .child("Waiting for approval"),
        )
        .when_some(code.on_cancel, |page, cancel| {
            page.child(
                Button::new("cancel-code")
                    .ghost()
                    .small()
                    .label("Cancel")
                    .on_click(move |event, window, cx| cancel(event, window, cx)),
            )
        })
        .into_any_element()
}

pub fn handler(run: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Handler {
    Rc::new(run)
}
