use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Sizable, v_flex};
use gpui_kit::{
    AnyElement, AppContext, ClipboardItem, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Subscription, Task, WeakEntity, Window, div,
};
use protocol::github::{Account, CancelSignIn, FinishSignIn, GetAccounts, SignInCode, StartSignIn};

use super::sign_in::{hostname, trusted, untrusted};
use super::{Onboarding, Step, parts};
use crate::notice;
use crate::runtime;

const GITHUB: &str = "github.com";

pub struct Github {
    onboarding: WeakEntity<Onboarding>,
    accounts: Option<Vec<Account>>,
    starting: Option<String>,
    prompt: Option<SignInCode>,
    enterprise: bool,
    host: Entity<InputState>,
    copied: bool,
    loading: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl Github {
    pub fn new(
        onboarding: WeakEntity<Onboarding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let host = cx.new(|cx| InputState::new(window, cx).placeholder("company.ghe.com"));
        let subscription = cx.subscribe_in(
            &host,
            window,
            |this: &mut Self, input, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let host = input.read(cx).value().trim().to_owned();
                    if !host.is_empty() && this.starting.is_none() {
                        this.begin(host, cx);
                    }
                }
            },
        );
        let machines = crate::state::store(cx).machines.clone();
        let waiting = cx.observe(&machines, |this: &mut Self, _, cx| {
            if this.accounts.is_none() && this.loading.is_none() {
                this.load(cx);
            }
        });
        let mut github = Self {
            onboarding,
            accounts: None,
            starting: None,
            prompt: None,
            enterprise: false,
            host,
            copied: false,
            loading: None,
            _subscriptions: vec![subscription, waiting],
        };
        github.load(cx);
        github
    }

    fn client(&self, cx: &Context<Self>) -> Option<client::Client> {
        let onboarding = self.onboarding.upgrade()?;
        onboarding
            .read(cx)
            .machine(cx)
            .map(|machine| machine.client)
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client(cx) else {
            return;
        };
        let request = runtime::call::<GetAccounts>(&client, (), cx);
        self.loading = Some(cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(accounts) => this.accounts = Some(accounts),
                    Err(error) => notice::error("Could not list your GitHub accounts", &error, cx),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn fail(&mut self, error: &dyn std::fmt::Display, cx: &mut Context<Self>) {
        self.starting = None;
        self.prompt = None;
        notice::error("Could not sign in to GitHub", &error.to_string(), cx);
        cx.notify();
    }

    fn begin(&mut self, host: String, cx: &mut Context<Self>) {
        let Some(client) = self.client(cx) else {
            return;
        };
        self.starting = Some(host.clone());
        self.copied = false;
        cx.notify();

        let start = runtime::call::<StartSignIn>(&client, host.clone(), cx);
        cx.spawn(async move |this, cx| {
            let prompt = match start.await {
                Ok(prompt) => prompt,
                Err(error) => {
                    this.update(cx, |this, cx| this.fail(&error, cx)).ok();
                    return;
                }
            };
            if !trusted(&prompt.url, &[&hostname(&host)]) {
                let cancel = cx.update(|cx| runtime::call::<CancelSignIn>(&client, (), cx));
                cancel.detach();
                this.update(cx, |this, cx| this.fail(&untrusted(&prompt.url), cx))
                    .ok();
                return;
            }

            let finish = cx.update(|cx| {
                cx.open_url(&prompt.url);
                runtime::call::<FinishSignIn>(&client, (), cx)
            });
            this.update(cx, |this, cx| {
                this.prompt = Some(prompt);
                cx.notify();
            })
            .ok();
            let result = finish.await;
            this.update(cx, |this, cx| {
                this.starting = None;
                this.prompt = None;
                match result {
                    Ok(()) => this.load(cx),
                    Err(error) => this.fail(&error, cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Github {
    fn code(prompt: SignInCode, copied: bool, cx: &mut Context<Self>) -> AnyElement {
        let code = prompt.code.clone();
        parts::device_code(
            parts::Code {
                value: prompt.code,
                url: prompt.url,
                site: "GitHub".into(),
                hint: None,
                copied,
                on_copy: parts::handler(cx.listener(move |this, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(code.clone()));
                    this.copied = true;
                    cx.notify();
                })),
                on_cancel: None,
            },
            cx,
        )
    }

    fn nav(empty: bool, cx: &mut Context<Self>) -> AnyElement {
        let back = parts::handler(cx.listener(|this, _, window, cx| {
            let server = this
                .onboarding
                .upgrade()
                .and_then(|onboarding| onboarding.read(cx).machine(cx))
                .is_some_and(|machine| !machine.local());
            let step = if server { Step::Server } else { Step::Where };
            this.onboarding
                .update(cx, |onboarding, cx| onboarding.go(step, None, window, cx))
                .ok();
        }));
        let next = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Agents, None, window, cx);
                })
                .ok();
        }));

        parts::nav(Some(back), "Continue", false, empty, next)
    }
}

impl Render for Github {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(prompt) = self.prompt.clone() {
            return Self::code(prompt, self.copied, cx);
        }

        let Some(accounts) = self.accounts.clone() else {
            return div().child(Spinner::new()).into_any_element();
        };
        let has_github = accounts.iter().any(|account| account.host == GITHUB);
        let has_enterprise = accounts.iter().any(|account| account.host != GITHUB);
        let starting_github = self.starting.as_deref() == Some(GITHUB);
        let starting_enterprise = self.starting.as_deref().is_some_and(|host| host != GITHUB);

        let mut rows = v_flex()
            .w_full()
            .gap_2()
            .child(parts::connection(
                parts::Connection {
                    id: "github".into(),
                    name: "GitHub".into(),
                    icon: None,
                    action: "Connect",
                    pending: "Connecting…",
                    connected: has_github,
                    busy: starting_github,
                    on_connect: parts::handler(cx.listener(|this, _, _, cx| {
                        if this.starting.is_none() {
                            this.begin(GITHUB.to_owned(), cx);
                        }
                    })),
                    on_cancel: None,
                },
                cx,
            ))
            .child(parts::connection(
                parts::Connection {
                    id: "enterprise".into(),
                    name: "GitHub Enterprise".into(),
                    icon: None,
                    action: "Connect",
                    pending: "Connecting…",
                    connected: has_enterprise,
                    busy: starting_enterprise,
                    on_connect: parts::handler(cx.listener(|this, _, window, cx| {
                        this.enterprise = true;
                        this.host.update(cx, |host, cx| host.focus(window, cx));
                        cx.notify();
                    })),
                    on_cancel: None,
                },
                cx,
            ));
        if self.enterprise && !has_enterprise {
            rows = rows.child(
                Input::new(&self.host).suffix(
                    Button::new("connect-enterprise")
                        .ghost()
                        .xsmall()
                        .loading(starting_enterprise)
                        .label(if starting_enterprise {
                            "Connecting…"
                        } else {
                            "Connect"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            let host = this.host.read(cx).value().trim().to_owned();
                            if !host.is_empty() && this.starting.is_none() {
                                this.begin(host, cx);
                            }
                        })),
                ),
            );
        }

        parts::section(
            "Set up GitHub",
            "Your agents use the GitHub CLI to clone, push and open pull requests.",
        )
        .child(rows)
        .child(Self::nav(accounts.is_empty(), cx))
        .into_any_element()
    }
}
