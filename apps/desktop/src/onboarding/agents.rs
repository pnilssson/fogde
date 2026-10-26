mod render;

use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{AppContext, Context, Entity, WeakEntity, Window};
use protocol::agents::{
    Agent, AgentPrompt, CancelAgentSignIn, FinishAgentSignIn, FinishAgentSignInParams,
    InstallAgent, StartAgentSignIn,
};

use super::Onboarding;
use super::sign_in::{trusted, untrusted};
use crate::notice;
use crate::runtime;
use crate::state::store;

struct Active {
    agent: Agent,
    prompt: Option<AgentPrompt>,
    submitting: bool,
}

pub struct Agents {
    onboarding: WeakEntity<Onboarding>,
    active: Option<Active>,
    attempt: u64,
    installing: Vec<Agent>,
    code: Entity<InputState>,
    copied: bool,
}

impl Agents {
    pub fn new(
        onboarding: WeakEntity<Onboarding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let code = cx.new(|cx| InputState::new(window, cx).placeholder("Paste the code"));
        let agents = Self {
            onboarding,
            active: None,
            attempt: 0,
            installing: Vec::new(),
            code,
            copied: false,
        };
        agents.reload(cx);
        let state = store(cx).agents.clone();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        agents
    }

    fn machine(&self, cx: &Context<Self>) -> Option<crate::state::machines::Machine> {
        self.onboarding.upgrade()?.read(cx).machine(cx)
    }

    fn reload(&self, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        store(cx)
            .agents
            .clone()
            .update(cx, |agents, cx| agents.reload(&machine.id, cx));
    }

    fn end(&mut self, attempt: u64, cx: &mut Context<Self>) {
        if self.attempt == attempt {
            self.attempt += 1;
            self.active = None;
            self.reload(cx);
            cx.notify();
        }
    }

    fn fail(&mut self, attempt: u64, agent: Agent, error: &str, cx: &mut Context<Self>) {
        if self.attempt == attempt {
            notice::error(
                format!("Could not connect {}", agent.spec().account),
                &error,
                cx,
            );
        }
        self.end(attempt, cx);
    }

    fn install(&mut self, agent: Agent, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        self.installing.push(agent);
        cx.notify();
        let request = runtime::call::<InstallAgent>(&machine.client, agent, cx);
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                this.installing.retain(|known| *known != agent);
                if let Err(error) = result {
                    notice::error(
                        format!("Could not install {}", agent.spec().account),
                        &error,
                        cx,
                    );
                }
                this.reload(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn begin(&mut self, agent: Agent, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        self.attempt += 1;
        let attempt = self.attempt;
        self.copied = false;
        self.active = Some(Active {
            agent,
            prompt: None,
            submitting: false,
        });
        cx.notify();

        let client = machine.client.clone();
        let start = runtime::call::<StartAgentSignIn>(&client, agent, cx);
        cx.spawn(async move |this, cx| {
            let prompt = match start.await {
                Ok(prompt) => prompt,
                Err(error) => {
                    this.update(cx, |this, cx| {
                        this.fail(attempt, agent, &error.to_string(), cx);
                    })
                    .ok();
                    return;
                }
            };
            let url = match &prompt {
                AgentPrompt::Done => {
                    this.update(cx, |this, cx| this.end(attempt, cx)).ok();
                    return;
                }
                AgentPrompt::Browser => None,
                AgentPrompt::Code { url, .. } | AgentPrompt::Paste { url } => Some(url.clone()),
            };
            if let Some(url) = &url
                && !trusted(url, agent.spec().domains)
            {
                cx.update(|cx| runtime::call::<CancelAgentSignIn>(&client, agent, cx))
                    .detach();
                this.update(cx, |this, cx| {
                    this.fail(attempt, agent, &untrusted(url), cx);
                })
                .ok();
                return;
            }

            let paste = matches!(prompt, AgentPrompt::Paste { .. });
            let shown = this
                .update(cx, |this, cx| {
                    if this.attempt != attempt {
                        return false;
                    }
                    if let Some(url) = &url {
                        cx.open_url(url);
                    }
                    this.active = Some(Active {
                        agent,
                        prompt: Some(prompt),
                        submitting: false,
                    });
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if shown && !paste {
                this.update(cx, |this, cx| this.finish(attempt, agent, None, cx))
                    .ok();
            }
        })
        .detach();
    }

    fn finish(&mut self, attempt: u64, agent: Agent, code: Option<String>, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        if let Some(active) = &mut self.active {
            active.submitting = true;
        }
        cx.notify();
        let request = runtime::call::<FinishAgentSignIn>(
            &machine.client,
            FinishAgentSignInParams { agent, code },
            cx,
        );
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| match result {
                Ok(()) => this.end(attempt, cx),
                Err(error) => this.fail(attempt, agent, &error.to_string(), cx),
            })
            .ok();
        })
        .detach();
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        let (Some(active), Some(machine)) = (
            self.active.as_ref().map(|active| active.agent),
            self.machine(cx),
        ) else {
            return;
        };
        let attempt = self.attempt;
        self.end(attempt, cx);
        let request = runtime::call::<CancelAgentSignIn>(&machine.client, active, cx);
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    notice::error(
                        format!("Could not cancel signing in to {}", active.spec().account),
                        &error,
                        cx,
                    );
                }
                this.reload(cx);
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn code(&self) -> &Entity<InputState> {
        &self.code
    }

    pub(super) fn paste(&mut self, cx: &mut Context<Self>) {
        let code = self.code.read(cx).value().trim().to_owned();
        let Some(agent) = self
            .active
            .as_ref()
            .filter(|active| !active.submitting)
            .map(|active| active.agent)
        else {
            return;
        };
        if !code.is_empty() {
            self.finish(self.attempt, agent, Some(code), cx);
        }
    }
}

pub(super) fn paste_button(submitting: bool) -> Button {
    Button::new("paste")
        .ghost()
        .xsmall()
        .loading(submitting)
        .label(if submitting {
            "Connecting…"
        } else {
            "Connect"
        })
}

pub(super) fn paste_input(input: &Entity<InputState>, button: Button) -> Input {
    Input::new(input).suffix(button)
}
