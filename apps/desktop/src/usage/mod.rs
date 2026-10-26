mod time;

use std::collections::HashMap;
use std::time::Duration;

use chrono::{Local, TimeZone};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Subscription, Task, Window, div, px,
    relative,
};
use protocol::agents::Agent;
use protocol::usage::{ListUsage, Span, Usage, UsageEvent, Window as Limit};

use crate::runtime;
use crate::state::listen::drain;
use crate::state::machines::{Change, Machines};
use crate::state::store;
use crate::theme::{self, Type};
use crate::workspaces::agent_icon;

const TICK: Duration = Duration::from_secs(30);

pub struct Meters {
    machines: Entity<Machines>,
    readings: HashMap<SharedString, Vec<Usage>>,
    watches: HashMap<SharedString, Task<()>>,
    now: i64,
    _tick: Task<()>,
    _subscriptions: Vec<Subscription>,
}

fn now() -> i64 {
    Local::now().timestamp_millis()
}

fn reset_time(window: &Limit, now: i64) -> String {
    let Some(date) = Local
        .timestamp_millis_opt(time::resets_at(window, now))
        .single()
    else {
        return String::new();
    };
    match window.span {
        Span::Month => date.format("%B %-d").to_string(),
        Span::Session | Span::Week => date.format("%A %-I:%M %p").to_string(),
    }
}

fn describe(window: &Limit, now: i64) -> String {
    format!(
        "{} limit {}% left, resets in {} ({})",
        time::span(window.span),
        time::left(window),
        time::until(time::resets_at(window, now) - now),
        reset_time(window, now)
    )
}

impl Meters {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = store(cx);
        let (machines, settings) = (state.machines.clone(), state.settings.clone());
        let subscriptions = vec![
            cx.subscribe(&machines, |this, _, change, cx| match change {
                Change::Connected(id) => this.load(id, cx),
                Change::Listed => this.sync(cx),
            }),
            cx.observe(&settings, |_, _, cx| cx.notify()),
        ];
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this
                    .update(cx, |this, cx| {
                        this.now = now();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let mut meters = Self {
            machines,
            readings: HashMap::new(),
            watches: HashMap::new(),
            now: now(),
            _tick: tick,
            _subscriptions: subscriptions,
        };
        meters.sync(cx);
        meters
    }

    fn sync(&mut self, cx: &mut Context<Self>) {
        let connected: Vec<_> = self
            .machines
            .read(cx)
            .connected()
            .map(|machine| (machine.id.clone(), machine.client.clone()))
            .collect();
        self.readings
            .retain(|id, _| connected.iter().any(|(known, _)| known == id));
        self.watches
            .retain(|id, _| connected.iter().any(|(known, _)| known == id));
        for (id, client) in connected {
            if self.watches.contains_key(&id) {
                continue;
            }
            let key = id.clone();
            let task = drain::<UsageEvent, _>(&client, cx, move |this: &mut Self, usage, _| {
                this.readings.insert(key.clone(), usage);
                true
            });
            self.watches.insert(id.clone(), task);
            self.load(&id, cx);
        }
    }

    fn load(&mut self, machine: &str, cx: &mut Context<Self>) {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return;
        };
        let request = runtime::call::<ListUsage>(&client, (), cx);
        let key = SharedString::from(machine.to_owned());
        cx.spawn(async move |this, cx| {
            if let Ok(usage) = request.await {
                this.update(cx, |this, cx| {
                    this.readings.insert(key, usage);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn meter(
        &self,
        agent: Agent,
        open: &[Limit],
        shown: Limit,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lines: Vec<String> = open
            .iter()
            .map(|window| describe(window, self.now))
            .collect();
        let left = time::left(&shown);
        let muted = cx.theme().muted_foreground;
        let track = theme::tokens(cx).hover;
        h_flex()
            .id(SharedString::from(format!(
                "usage-{}",
                agent.spec().command
            )))
            .gap_2()
            .px_2()
            .h_8()
            .rounded_md()
            .text_color(muted)
            .child(agent_icon(agent, cx))
            .child(
                div().w_12().h_1().rounded_full().bg(track).child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(muted)
                        .w(relative(crate::num::real(left) / 100.)),
                ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .type_caption()
                    .font_family(theme::MONO)
                    .child(format!("{left}%"))
                    .child(time::until(time::resets_at(&shown, self.now) - self.now)),
            )
            .tooltip(move |window, cx| {
                let lines = lines.clone();
                Tooltip::element(move |_, _| {
                    v_flex().children(lines.iter().map(|line| div().child(line.clone())))
                })
                .build(window, cx)
            })
            .into_any_element()
    }
}

impl Render for Meters {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(shown) = store(cx).settings.read(cx).shown.clone() else {
            return div().into_any_element();
        };
        let readings: Vec<Usage> = self.readings.values().flatten().cloned().collect();
        let meters: Vec<AnyElement> = Agent::ALL
            .into_iter()
            .filter(|agent| shown.shows(*agent))
            .filter_map(|agent| {
                let usage = time::newest(&readings, agent)?;
                let (open, current) = time::shown(usage, self.now)?;
                Some((agent, open, current))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(agent, open, current)| self.meter(agent, &open, current, cx))
            .collect();
        h_flex()
            .gap_2()
            .min_w(px(0.))
            .children(meters)
            .into_any_element()
    }
}
