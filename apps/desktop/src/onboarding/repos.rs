use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Sizable, v_flex};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    WeakEntity, Window, div, prelude::FluentBuilder,
};
use protocol::repos::{CheckReposFolder, GetReposFolder, SetReposFolder};

use super::{Onboarding, Step, parts};
use crate::folders;
use crate::runtime;
use crate::state::machines::Machine;
use crate::theme::Type;

pub struct Repos {
    onboarding: WeakEntity<Onboarding>,
    folder: Entity<InputState>,
    error: Option<String>,
    saving: bool,
    loaded: bool,
    _subscriptions: Vec<Subscription>,
}

impl Repos {
    pub fn new(
        onboarding: WeakEntity<Onboarding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let folder = cx.new(|cx| InputState::new(window, cx));
        let subscriptions = vec![cx.subscribe_in(
            &folder,
            window,
            |this: &mut Self, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.save(window, cx),
                InputEvent::Change => {
                    this.error = None;
                    cx.notify();
                }
                _ => {}
            },
        )];
        let mut repos = Self {
            onboarding,
            folder,
            error: None,
            saving: false,
            loaded: false,
            _subscriptions: subscriptions,
        };
        repos.load(window, cx);
        repos
    }

    fn machine(&self, cx: &App) -> Option<Machine> {
        self.onboarding.upgrade()?.read(cx).machine(cx)
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        let request = runtime::call::<GetReposFolder>(&machine.client, (), cx);
        cx.spawn_in(window, async move |this, cx| {
            let folder = request.await.unwrap_or_default();
            this.update_in(cx, |this, window, cx| {
                this.loaded = true;
                this.folder
                    .update(cx, |input, cx| input.set_value(folder, window, cx));
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let folder = self.folder.read(cx).value().trim().to_owned();
        if folder.is_empty() {
            self.error = Some("Enter a folder".into());
            cx.notify();
            return;
        }
        let Some(machine) = self.machine(cx) else {
            return;
        };
        self.saving = true;
        cx.notify();

        let client = machine.client.clone();
        let check = runtime::call::<CheckReposFolder>(&client, folder.clone(), cx);
        cx.spawn_in(window, async move |this, cx| {
            if let Err(error) = check.await {
                this.update(cx, |this, cx| {
                    this.saving = false;
                    this.error = Some(error.to_string());
                    cx.notify();
                })
                .ok();
                return;
            }
            let Ok(save) = cx.update(|_, cx| runtime::call::<SetReposFolder>(&client, folder, cx))
            else {
                return;
            };
            let result = save.await;
            this.update_in(cx, |this, window, cx| {
                this.saving = false;
                cx.notify();
                match result {
                    Ok(()) => {
                        this.onboarding
                            .update(cx, |onboarding, cx| {
                                onboarding.go(Step::Clone, None, window, cx);
                            })
                            .ok();
                    }
                    Err(error) => crate::notice::error("Could not save the folder", &error, cx),
                }
            })
            .ok();
        })
        .detach();
    }

    fn browse(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        let start = self.folder.read(cx).value().to_string();
        let me = cx.entity().downgrade();
        if machine.local() {
            cx.spawn_in(window, async move |this, cx| {
                let mut dialog = rfd::AsyncFileDialog::new();
                if !start.is_empty() {
                    dialog = dialog.set_directory(&start);
                }
                let Some(picked) = dialog.pick_folder().await else {
                    return;
                };
                let folder = picked.path().display().to_string();
                this.update_in(cx, |this, window, cx| {
                    this.folder
                        .update(cx, |input, cx| input.set_value(folder, window, cx));
                })
                .ok();
            })
            .detach();
            return;
        }

        let choose = Rc::new(move |path: String, _, window: &mut Window, cx: &mut App| {
            me.update(cx, |this, cx| {
                this.folder
                    .update(cx, |input, cx| input.set_value(path, window, cx));
            })
            .ok();
            gpui_kit::component::WindowExt::close_dialog(window, cx);
        });
        folders::open(
            machine.client,
            start,
            "Choose a folder",
            "Use this folder",
            choose,
            window,
            cx,
        );
    }
}

impl Render for Repos {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let back = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Agents, None, window, cx);
                })
                .ok();
        }));
        let next = parts::handler(cx.listener(|this, _, window, cx| this.save(window, cx)));

        parts::section(
            "Choose a repos folder",
            "Repositories are cloned here. Pick a folder you already use to keep your existing clones.",
        )
        .child(
            v_flex()
                .w_full()
                .gap_1()
                .child(Input::new(&self.folder).suffix(
                    Button::new("browse")
                        .ghost()
                        .xsmall()
                        .label("Browse")
                        .on_click(cx.listener(|this, _, window, cx| this.browse(window, cx))),
                ))
                .when_some(self.error.clone(), |field, error| {
                    field.child(div().type_caption().text_color(cx.theme().danger).child(error))
                }),
        )
        .child(parts::nav(Some(back), if self.saving { "Saving…" } else { "Continue" }, self.saving, !self.loaded, next))
    }
}
