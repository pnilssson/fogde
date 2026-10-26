use super::Agents;
use crate::onboarding::{Step, parts};
use crate::state::store;
use crate::workspaces::agent_icon;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::v_flex;
use gpui_kit::{ClipboardItem, Context, IntoElement, ParentElement, Render, Styled, Window, div};
use protocol::agents::{AgentPrompt, AgentState};

impl Agents {
    fn prompt_page(&self, cx: &mut Context<Self>) -> Option<gpui_kit::AnyElement> {
        let active = self.active.as_ref()?;
        let account = active.agent.spec().account;
        let cancel = parts::handler(cx.listener(|this, _, _, cx| this.cancel(cx)));
        match active.prompt.clone()? {
            AgentPrompt::Code { code, url } => {
                let copy = code.clone();
                Some(parts::device_code(
                    parts::Code {
                        value: code,
                        url,
                        site: account.into(),
                        hint: Some(format!(
                            "Device code sign-in needs to be on in your {account} security settings."
                        )),
                        copied: self.copied,
                        on_copy: parts::handler(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()));
                            this.copied = true;
                            cx.notify();
                        })),
                        on_cancel: Some(cancel),
                    },
                    cx,
                ))
            }
            AgentPrompt::Paste { url } => Some(crate::onboarding::agents_paste::page(
                self,
                account,
                url,
                active.submitting,
                cancel,
                cx,
            )),
            AgentPrompt::Browser | AgentPrompt::Done => None,
        }
    }
}

impl Render for Agents {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(page) = self.prompt_page(cx) {
            return page;
        }
        let Some(machine) = self.machine(cx) else {
            return div().child(Spinner::new()).into_any_element();
        };
        let Some(statuses) = store(cx)
            .agents
            .read(cx)
            .list(&machine.id)
            .map(<[_]>::to_vec)
        else {
            return div().child(Spinner::new()).into_any_element();
        };

        let rows = statuses.iter().map(|status| {
            let agent = status.agent;
            let missing = matches!(status.state, AgentState::Missing);
            let signing = self
                .active
                .as_ref()
                .is_some_and(|active| active.agent == agent);
            let busy = if missing {
                self.installing.contains(&agent)
            } else {
                signing
            };
            parts::connection(
                parts::Connection {
                    id: agent.spec().command.into(),
                    name: agent.spec().account.into(),
                    icon: Some(agent_icon(agent, cx)),
                    action: if missing { "Install" } else { "Connect" },
                    pending: if missing {
                        "Installing…"
                    } else {
                        "Connecting…"
                    },
                    connected: matches!(status.state, AgentState::SignedIn),
                    busy,
                    on_connect: parts::handler(cx.listener(move |this, _, _, cx| {
                        if missing {
                            this.install(agent, cx);
                        } else if this.active.is_none() {
                            this.begin(agent, cx);
                        }
                    })),
                    on_cancel: (signing && !missing)
                        .then(|| parts::handler(cx.listener(|this, _, _, cx| this.cancel(cx)))),
                },
                cx,
            )
        });
        let signed_in = statuses
            .iter()
            .any(|status| matches!(status.state, AgentState::SignedIn));
        let back = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Github, None, window, cx);
                })
                .ok();
        }));
        let next = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Repos, None, window, cx);
                })
                .ok();
        }));

        parts::section(
            "Connect your agents",
            "Your coding agents run on your own plans. Sign in to at least one.",
        )
        .child(v_flex().w_full().gap_2().children(rows))
        .child(parts::nav(Some(back), "Continue", false, !signed_in, next))
        .into_any_element()
    }
}
