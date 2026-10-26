use std::collections::{HashMap, HashSet};

use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window, div, px,
};
use protocol::github::GetAccounts;
use protocol::repos::{CloneRepo, ListRepos, RemoteRepo, Repo};

use super::{Onboarding, Step, parts};
use crate::assets::Glyph;
use crate::notice;
use crate::runtime;
use crate::state::machines::Machine;
use crate::theme::Type;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Progress {
    Cloning,
    Cloned,
}

fn key(repo: &RemoteRepo) -> String {
    format!("{}/{}/{}", repo.host, repo.owner, repo.name)
}

pub struct CloneRepos {
    onboarding: WeakEntity<Onboarding>,
    hosts: Option<Vec<String>>,
    repos: HashMap<String, Vec<RemoteRepo>>,
    progress: HashMap<String, Progress>,
    filter: Entity<InputState>,
}

impl CloneRepos {
    pub fn new(
        onboarding: WeakEntity<Onboarding>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx).placeholder("Filter repos"));
        cx.observe(&filter, |_, _, cx| cx.notify()).detach();
        let page = Self {
            onboarding,
            hosts: None,
            repos: HashMap::new(),
            progress: HashMap::new(),
            filter,
        };
        page.load(cx);
        page
    }

    fn machine(&self, cx: &App) -> Option<Machine> {
        self.onboarding.upgrade()?.read(cx).machine(cx)
    }

    fn load(&self, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        let client = machine.client.clone();
        let accounts = runtime::call::<GetAccounts>(&client, (), cx);
        cx.spawn(async move |this, cx| {
            let hosts: Vec<String> = accounts
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|account| account.host)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            this.update(cx, |this, cx| {
                this.hosts = Some(hosts.clone());
                cx.notify();
            })
            .ok();
            for host in hosts {
                let request = cx.update(|cx| runtime::call::<ListRepos>(&client, host.clone(), cx));
                let result = request.await;
                this.update(cx, |this, cx| {
                    match result {
                        Ok(repos) => {
                            this.repos.insert(host, repos);
                        }
                        Err(error) => notice::error(
                            format!("Could not list your repos on {host}"),
                            &error,
                            cx,
                        ),
                    }
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn clone_repo(&mut self, repo: &RemoteRepo, cx: &mut Context<Self>) {
        let Some(machine) = self.machine(cx) else {
            return;
        };
        let id = key(repo);
        self.progress.insert(id.clone(), Progress::Cloning);
        cx.notify();
        let params = Repo {
            host: repo.host.clone(),
            owner: repo.owner.clone(),
            name: repo.name.clone(),
        };
        let name = repo.name.clone();
        let request = runtime::call::<CloneRepo>(&machine.client, params, cx);
        cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.progress.insert(id, Progress::Cloned);
                    }
                    Err(error) => {
                        this.progress.remove(&id);
                        notice::error(format!("Could not clone {name}"), &error, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn list(&self, cx: &mut Context<Self>) -> AnyElement {
        let hosts = self.hosts.clone().unwrap_or_default();
        let search = self.filter.read(cx).value().trim().to_lowercase();
        let several = hosts.len() > 1;
        let mut list = v_flex().gap_4().pr_4();
        let mut any = false;

        for host in &hosts {
            let repos: Vec<RemoteRepo> = self
                .repos
                .get(host)
                .map(|repos| {
                    repos
                        .iter()
                        .filter(|repo| {
                            format!("{}/{}", repo.owner, repo.name)
                                .to_lowercase()
                                .contains(&search)
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            if repos.is_empty() {
                continue;
            }
            any = true;
            let mut group = v_flex().gap_2();
            if several {
                group = group.child(
                    div()
                        .type_caption()
                        .text_color(cx.theme().muted_foreground)
                        .child(host.clone()),
                );
            }
            for repo in repos {
                let status = if repo.cloned {
                    Some(Progress::Cloned)
                } else {
                    self.progress.get(&key(&repo)).copied()
                };
                let target = repo.clone();
                group = group.child(
                    h_flex()
                        .gap_2()
                        .child(
                            Checkbox::new(SharedString::from(key(&repo)))
                                .checked(status.is_some())
                                .disabled(status.is_some())
                                .label(format!("{}/{}", repo.owner, repo.name))
                                .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                                    if *checked {
                                        this.clone_repo(&target, cx);
                                    }
                                })),
                        )
                        .child(match status {
                            Some(Progress::Cloning) => h_flex()
                                .gap_1()
                                .type_caption()
                                .child(Spinner::new().xsmall())
                                .child("Cloning")
                                .into_any_element(),
                            Some(Progress::Cloned) => parts::done("Cloned", cx),
                            None => div().into_any_element(),
                        }),
                );
            }
            list = list.child(group);
        }
        if !any {
            list = list.child(div().type_body().child(if self.hosts.is_none() {
                "Loading…"
            } else {
                "No repositories found"
            }));
        }
        div()
            .id("repos")
            .h(px(256.))
            .w_full()
            .overflow_y_scroll()
            .child(list)
            .into_any_element()
    }
}

impl Render for CloneRepos {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let finishing = self
            .onboarding
            .upgrade()
            .is_some_and(|onboarding| onboarding.read(cx).finishing());
        let back = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| {
                    onboarding.go(Step::Repos, None, window, cx);
                })
                .ok();
        }));
        let next = parts::handler(cx.listener(|this, _, window, cx| {
            this.onboarding
                .update(cx, |onboarding, cx| onboarding.finish(window, cx))
                .ok();
        }));

        parts::section(
            "Clone your repos",
            "Pick the repos you work on. They are cloned into your repos folder.",
        )
        .w(px(384.))
        .child(Input::new(&self.filter).prefix(Icon::new(Glyph::MagnifyingGlass).small()))
        .child(self.list(cx))
        .child(parts::nav(
            Some(back),
            if finishing {
                "Finishing…"
            } else {
                "Continue"
            },
            finishing,
            false,
            next,
        ))
    }
}
