use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, TitleBar, h_flex, v_flex};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, Task, Window, div, prelude::FluentBuilder, px,
};
use protocol::onboarding::GetOnboarded;

use crate::assets::Glyph;
use crate::notice;
use crate::onboarding::Onboarding;
use crate::runtime;
use crate::state::machines::Machines;
use crate::state::store;
use crate::theme::{self, Type};
use crate::window::{self, BAR};
use crate::workbench::{self, Workbench};

struct Current(gpui_kit::WeakEntity<Fogde>);

impl gpui_kit::Global for Current {}

pub fn current(cx: &gpui_kit::App) -> Option<Entity<Fogde>> {
    cx.try_global::<Current>()
        .and_then(|current| current.0.upgrade())
}

enum Screen {
    Loading,
    Onboarding(Entity<Onboarding>),
    Workbench(Entity<Workbench>),
}

pub struct Fogde {
    screen: Screen,
    onboarded: Option<bool>,
    failure: Option<SharedString>,
    checking: Option<Task<()>>,
    _attention: Entity<crate::attention::Attention>,
    _network: crate::state::network::Network,
    _subscriptions: Vec<Subscription>,
}

impl Fogde {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = store(cx);
        let machines = state.machines.clone();
        let settings = state.settings.clone();
        let subscriptions = vec![
            cx.subscribe_in(&notice::notices(cx), window, |_, _, notice, window, cx| {
                notice::present(notice, window, cx);
            }),
            cx.observe_in(&machines, window, |this, _, window, cx| {
                this.route(window, cx);
            }),
            cx.observe(&settings, |_, settings, cx| {
                let (choice, material) = {
                    let settings = settings.read(cx);
                    (settings.theme, settings.material)
                };
                if let Some(choice) = choice.filter(|choice| *choice != theme::choice(cx)) {
                    theme::choose(choice, cx);
                }
                if let Some(material) = material.filter(|material| *material != theme::material(cx))
                {
                    window::store_material(material, cx);
                }
            }),
            cx.observe_window_bounds(window, |_, window, cx| window::remember(window, cx)),
            cx.observe_window_appearance(window, |_, _, cx| theme::apply(cx)),
            cx.observe_window_activation(window, |_, window, cx| {
                if window.is_window_active() {
                    let machines = store(cx).machines.clone();
                    machines.update(cx, |machines, cx| machines.retry(cx));
                }
            }),
        ];

        let me = cx.entity().downgrade();
        cx.set_global(Current(me));
        let mut app = Self {
            screen: Screen::Loading,
            onboarded: None,
            failure: None,
            checking: None,
            _attention: crate::attention::init(window, cx),
            _network: crate::state::network::watch(&store(cx).machines.clone(), cx),
            _subscriptions: subscriptions,
        };
        app.check(window, cx);
        app
    }

    pub fn check(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let own = store(cx).machines.read(cx).own().clone();
        let request = runtime::call::<GetOnboarded>(&own, (), cx);
        self.checking = Some(cx.spawn_in(window, async move |this, cx| {
            let onboarded = request.await;
            this.update_in(cx, |this, window, cx| {
                match onboarded {
                    Ok(onboarded) => this.onboarded = Some(onboarded),
                    Err(error) => this.failure = Some(error.to_string().into()),
                }
                this.route(window, cx);
                cx.notify();
            })
            .ok();
        }));
    }

    fn retry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let machines = store(cx).machines.clone();
        if machines.read(cx).failure().is_some() {
            machines.update(cx, Machines::load);
        }
        if self.failure.take().is_some() {
            self.check(window, cx);
        }
        cx.notify();
    }

    fn loading(&self, cx: &mut Context<Self>) -> AnyElement {
        let failure = self
            .failure
            .clone()
            .or_else(|| store(cx).machines.read(cx).failure().cloned());
        let Some(failure) = failure else {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Spinner::new())
                .into_any_element();
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .max_w(px(384.))
                    .items_start()
                    .gap_4()
                    .child(div().type_display().child("Something went wrong"))
                    .child(div().type_body().child(failure))
                    .child(
                        Button::new("retry")
                            .primary()
                            .small()
                            .label("Try again")
                            .on_click(cx.listener(|this, _, window, cx| this.retry(window, cx))),
                    ),
            )
            .into_any_element()
    }

    fn route(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let machines = store(cx).machines.read(cx);
        let Some(onboarded) = self.onboarded.filter(|_| machines.loaded()) else {
            return;
        };
        let count = machines.all().len();
        let only = (count == 1).then(|| machines.all()[0].id.to_string());

        if onboarded && count > 0 {
            if !matches!(self.screen, Screen::Workbench(_)) {
                self.screen = Screen::Workbench(cx.new(|cx| Workbench::new(window, cx)));
            }
        } else if !matches!(self.screen, Screen::Onboarding(_)) {
            let me = cx.entity().downgrade();
            self.screen =
                Screen::Onboarding(cx.new(|cx| Onboarding::new(me, only, onboarded, window, cx)));
        }
        cx.notify();
    }

    pub fn add_machine(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let me = cx.entity().downgrade();
        self.screen = Screen::Onboarding(cx.new(|cx| Onboarding::new(me, None, true, window, cx)));
        cx.notify();
    }

    pub fn finish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.onboarded = Some(true);
        self.screen = Screen::Loading;
        let machines = store(cx).machines.clone();
        machines.update(cx, Machines::load);
        self.route(window, cx);
    }

    fn title_bar(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let toggle = match &self.screen {
            Screen::Workbench(workbench) => {
                let open = workbench.read(cx).sidebar_open(window);
                let label = if open { "Hide sidebar" } else { "Show sidebar" };
                Some(
                    Button::new("sidebar-toggle")
                        .ghost()
                        .small()
                        .icon(Icon::new(if open {
                            Glyph::SidebarSimpleFill
                        } else {
                            Glyph::SidebarSimple
                        }))
                        .tooltip(label)
                        .on_click(|_, window, cx| workbench::toggle(window, cx)),
                )
            }
            _ => None,
        };
        if cfg!(target_os = "linux") && toggle.is_none() {
            return None;
        }

        Some(
            TitleBar::new()
                .h(BAR)
                .bg(gpui_kit::transparent_black())
                .border_b_0()
                .when(cfg!(target_os = "macos"), Styled::pl_20)
                .when(!cfg!(target_os = "macos"), Styled::pl_2)
                .child(h_flex().gap_2().children(toggle))
                .into_any_element(),
        )
    }
}

impl Render for Fogde {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content: AnyElement = match &self.screen {
            Screen::Loading => self.loading(cx),
            Screen::Onboarding(onboarding) => onboarding.clone().into_any_element(),
            Screen::Workbench(workbench) => workbench.clone().into_any_element(),
        };

        v_flex()
            .size_full()
            .text_color(cx.theme().foreground)
            .font_family(theme::SANS)
            .children(self.title_bar(window, cx))
            .child(div().flex_1().min_h_0().child(content))
    }
}
