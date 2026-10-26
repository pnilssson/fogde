mod render;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::{Disableable, Sizable, v_flex};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, IntoElement, ParentElement, Styled, Subscription,
    Task, WeakEntity, Window, div,
};
use protocol::server::{CheckServer, ListSshHosts, ServerCheck, ServerCheckParams, SetUpServer};

use super::{Onboarding, Step};
use crate::notice;
use crate::runtime;
use crate::state::machines::Machines;
use crate::state::store;
use crate::theme::{self, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Connecting,
    SettingUp,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Connecting => "Connecting…",
            Phase::SettingUp => "Setting up…",
        }
    }
}

pub struct Server {
    onboarding: WeakEntity<Onboarding>,
    target: Entity<InputState>,
    hosts: Vec<String>,
    invalid: bool,
    phase: Option<Phase>,
    problem: Option<(String, ServerCheck)>,
    _hosts: Task<()>,
    _subscription: Subscription,
}

impl Server {
    pub fn new(
        onboarding: WeakEntity<Onboarding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let target = cx.new(|cx| InputState::new(window, cx).placeholder("root@203.0.113.7"));
        let subscription = cx.subscribe_in(
            &target,
            window,
            |this: &mut Self, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.submit(window, cx),
                InputEvent::Change => {
                    this.invalid = false;
                    this.problem = None;
                    cx.notify();
                }
                _ => {}
            },
        );
        let own = store(cx).machines.read(cx).own().clone();
        let request = runtime::call::<ListSshHosts>(&own, (), cx);
        let hosts = cx.spawn(async move |this, cx| {
            if let Ok(hosts) = request.await {
                this.update(cx, |this, cx| {
                    this.hosts = hosts;
                    cx.notify();
                })
                .ok();
            }
        });
        Self {
            onboarding,
            target,
            hosts: Vec::new(),
            invalid: false,
            phase: None,
            problem: None,
            _hosts: hosts,
            _subscription: subscription,
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.target.read(cx).value().trim().to_owned();
        if target.is_empty() {
            self.invalid = true;
            cx.notify();
            return;
        }
        self.problem = None;
        self.connect(target, None, window, cx);
    }

    fn connect(
        &mut self,
        target: String,
        trust: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.phase.is_some() {
            return;
        }
        self.phase = Some(Phase::Connecting);
        cx.notify();

        let own = store(cx).machines.read(cx).own().clone();
        let check = runtime::call::<CheckServer>(
            &own,
            ServerCheckParams {
                target: target.clone(),
                trust,
            },
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let checked = check.await;
            let ready = this
                .update(cx, |this, cx| match checked {
                    Ok(ServerCheck::Ready) => {
                        this.phase = Some(Phase::SettingUp);
                        cx.notify();
                        true
                    }
                    Ok(problem) => {
                        this.phase = None;
                        this.problem = Some((target.clone(), problem));
                        cx.notify();
                        false
                    }
                    Err(error) => {
                        this.phase = None;
                        notice::error(format!("Could not connect to {target}"), &error, cx);
                        cx.notify();
                        false
                    }
                })
                .unwrap_or(false);
            if !ready {
                return;
            }

            let setup = cx.update(|_, cx| runtime::call::<SetUpServer>(&own, target.clone(), cx));
            let Ok(setup) = setup else {
                return;
            };
            let result = setup.await;
            this.update_in(cx, |this, window, cx| {
                this.phase = None;
                cx.notify();
                match result {
                    Ok(machine) => {
                        let machines = store(cx).machines.clone();
                        machines.update(cx, Machines::load);
                        this.onboarding
                            .update(cx, |onboarding, cx| {
                                onboarding.go(Step::Github, Some(machine), window, cx);
                            })
                            .ok();
                    }
                    Err(error) => {
                        notice::error(format!("Could not connect to {target}"), &error, cx);
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    fn problem(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (target, problem) = self.problem.clone()?;
        let pending = self.phase.map(Phase::label);
        let (title, lines, action, trust): (String, Vec<String>, &str, Option<String>) =
            match problem {
                ServerCheck::Ready => return None,
                ServerCheck::Untrusted { fingerprint } => (
                    format!("First connection to {target}"),
                    vec![format!("Its fingerprint is {fingerprint}.")],
                    "Trust this server",
                    Some(fingerprint),
                ),
                ServerCheck::KeyRejected { keys } => {
                    let mut lines = if keys.is_empty() {
                        vec!["Sign-in works with the same keys as ssh on this computer: key files in your .ssh folder, or keys from an SSH agent. If your key lives in a password manager, turn on its SSH agent.".to_owned()]
                    } else {
                        let mut lines = vec!["ssh offered:".to_owned()];
                        lines.extend(
                            keys.iter()
                                .map(|key| format!("{} ({})", key.name, key.fingerprint)),
                        );
                        lines
                    };
                    lines.push("The server needs the key's public half. Most providers add it when you create or rebuild a server.".to_owned());
                    let title = if keys.is_empty() {
                        "This computer has no SSH key to sign in with".to_owned()
                    } else {
                        format!("{target} didn't accept your SSH key")
                    };
                    (title, lines, "Try again", None)
                }
            };

        let tokens = theme::tokens(cx);
        Some(
            v_flex()
                .w_full()
                .gap_2()
                .p_4()
                .rounded_lg()
                .bg(tokens.surface[2])
                .shadow(tokens.shadows[1].clone())
                .child(div().type_title().child(title))
                .children(lines.into_iter().map(|line| div().type_body().child(line)))
                .child(
                    Button::new("problem")
                        .secondary()
                        .small()
                        .loading(pending.is_some())
                        .disabled(pending.is_some())
                        .label(pending.unwrap_or(action))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.connect(target.clone(), trust.clone(), window, cx);
                        })),
                )
                .into_any_element(),
        )
    }
}
