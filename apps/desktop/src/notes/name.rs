use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Sizable, WindowExt, h_flex, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Task, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::Type;

type Submit = Rc<dyn Fn(String, &mut Window, &mut App) -> Task<bool>>;

pub struct NameForm {
    name: Entity<InputState>,
    action: &'static str,
    busy: &'static str,
    submitting: bool,
    invalid: bool,
    submit: Submit,
    _subscription: Subscription,
}

impl NameForm {
    fn go(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.invalid = true;
            cx.notify();
            return;
        }
        if self.submitting {
            return;
        }
        self.submitting = true;
        cx.notify();
        let task = (self.submit)(name, window, cx);
        cx.spawn_in(window, async move |this, cx| {
            let done = task.await;
            this.update_in(cx, |this, window, cx| {
                this.submitting = false;
                if done {
                    window.close_dialog(cx);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Render for NameForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(div().type_body().child("Name"))
                    .child(Input::new(&self.name))
                    .when(self.invalid, |field| {
                        field.child(
                            div()
                                .type_caption()
                                .text_color(cx.theme().danger)
                                .child("Enter a name"),
                        )
                    }),
            )
            .child(
                h_flex().justify_end().child(
                    Button::new("submit")
                        .primary()
                        .small()
                        .loading(self.submitting)
                        .label(if self.submitting {
                            self.busy
                        } else {
                            self.action
                        })
                        .on_click(cx.listener(|this, _, window, cx| this.go(window, cx))),
                ),
            )
    }
}

pub struct Prompt {
    pub title: SharedString,
    pub action: &'static str,
    pub busy: &'static str,
    pub placeholder: &'static str,
    pub initial: String,
}

pub fn ask(
    prompt: Prompt,
    window: &mut Window,
    cx: &mut App,
    submit: impl Fn(String, &mut Window, &mut App) -> Task<bool> + 'static,
) {
    let Prompt {
        title,
        action,
        busy,
        placeholder,
        initial,
    } = prompt;
    let form = cx.new(|cx| {
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(initial)
        });
        let subscription = cx.subscribe_in(
            &name,
            window,
            |this: &mut NameForm, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.go(window, cx),
                InputEvent::Change => {
                    this.invalid = false;
                    cx.notify();
                }
                _ => {}
            },
        );
        NameForm {
            name,
            action,
            busy,
            submitting: false,
            invalid: false,
            submit: Rc::new(submit),
            _subscription: subscription,
        }
    });
    let input = form.read(cx).name.clone();
    input.update(cx, |input, cx| input.focus(window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(420.)).child(form.clone())
    });
}
