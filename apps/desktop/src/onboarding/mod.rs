mod agents;
mod agents_paste;
mod clone;
mod github;
mod parts;
mod repos;
mod server;
pub mod sign_in;
mod step;
mod where_;

use std::time::Duration;

use gpui_kit::component::{WindowExt, v_flex};
use gpui_kit::{
    Animation, AnimationExt, AnyElement, AnyView, App, AppContext, Context, ElementId, IntoElement,
    ParentElement, Render, Styled, WeakEntity, Window, div, ease_out_quint, px,
};
use protocol::onboarding::FinishOnboarding;

pub use step::Step;

use crate::app::Fogde;
use crate::notice;
use crate::runtime;
use crate::state::machines::{LOCAL, Machine};
use crate::state::store;

pub struct Onboarding {
    app: WeakEntity<Fogde>,
    step: Step,
    machine: Option<String>,
    onboarded: bool,
    page: AnyView,
    turn: u64,
    forward: bool,
    finishing: bool,
}

impl Onboarding {
    pub fn new(
        app: WeakEntity<Fogde>,
        machine: Option<String>,
        onboarded: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let machines: Vec<String> = machine.into_iter().collect();
        let (step, machine) = step::first(onboarded, &machines);
        let mut onboarding = Self {
            app,
            step,
            machine: None,
            onboarded,
            page: cx.new(|_| crate::workspaces::desk::split::Blank).into(),
            turn: 0,
            forward: true,
            finishing: false,
        };
        onboarding.go(step, machine, window, cx);
        onboarding
    }

    pub fn machine(&self, cx: &App) -> Option<Machine> {
        let id = self.machine.as_deref()?;
        store(cx).machines.read(cx).get(id).cloned()
    }

    pub fn onboarded(&self) -> bool {
        self.onboarded
    }

    pub fn finishing(&self) -> bool {
        self.finishing
    }

    pub fn go(
        &mut self,
        step: Step,
        machine: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.clear_notifications(cx);
        if machine.is_some() {
            self.machine = machine;
        }
        let me = cx.entity().downgrade();
        let page = match step {
            Step::Where => cx.new(|cx| where_::Where::new(me, cx)).into(),
            Step::Server => cx.new(|cx| server::Server::new(me, window, cx)).into(),
            Step::Github => cx.new(|cx| github::Github::new(me, window, cx)).into(),
            Step::Agents => cx.new(|cx| agents::Agents::new(me, window, cx)).into(),
            Step::Repos => cx.new(|cx| repos::Repos::new(me, window, cx)).into(),
            Step::Clone => cx.new(|cx| clone::CloneRepos::new(me, window, cx)).into(),
        };
        self.forward = step::forward(self.step, step);
        self.turn += 1;
        self.step = step;
        self.page = page;
        cx.notify();
    }

    pub fn computer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.go(Step::Github, Some(LOCAL.to_owned()), window, cx);
    }

    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(app) = self.app.upgrade() {
            app.update(cx, |app, cx| app.finish(window, cx));
        }
    }

    pub fn finish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finishing = true;
        cx.notify();
        let own = store(cx).machines.read(cx).own().clone();
        let request = runtime::call::<FinishOnboarding>(&own, (), cx);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.finishing = false;
                match result {
                    Ok(()) => this.leave(window, cx),
                    Err(error) => notice::error("Could not finish setting up", &error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn shown(&self) -> Vec<Step> {
        let server = self.step == Step::Server
            || self
                .machine
                .as_deref()
                .is_some_and(|machine| machine != LOCAL);
        step::steps(server)
    }
}

impl Onboarding {
    fn page(&self, cx: &App) -> AnyElement {
        let page = div().relative().child(self.page.clone());
        if self.turn <= 1 {
            return page.into_any_element();
        }

        let offset = if self.forward { 24. } else { -24. };
        let still = cx.reduce_motion();
        let animation = Animation::new(Duration::from_millis(200)).with_easing(ease_out_quint());
        page.with_animation(
            ElementId::NamedInteger("page".into(), self.turn),
            animation,
            move |page, delta| {
                let page = page.opacity(delta);
                if still {
                    return page;
                }

                page.left(px(offset * (1. - delta)))
            },
        )
        .into_any_element()
    }
}

impl Render for Onboarding {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shown = self.shown();
        let position = shown
            .iter()
            .position(|step| *step == self.step)
            .unwrap_or_default();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .child(
                v_flex()
                    .items_start()
                    .gap_4()
                    .child(parts::bars(shown.len(), position, cx))
                    .child(self.page(cx)),
            )
    }
}
