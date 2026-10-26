use gpui_kit::component::IndexPath;
use gpui_kit::component::button::Button;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Icon, Selectable, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div, px,
};
use protocol::accessibility::Accessibility;
use protocol::agents::Agent;
use protocol::shell::Shell;
use protocol::stats::Stats;
use protocol::theme::{Material, Theme};

use super::{notes::NotesFolders, row, section};
use crate::assets::Glyph;
use crate::state::settings::Settings;
use crate::state::store;
use crate::theme;

type Toggle = fn(&mut Stats) -> &mut bool;

const SYSTEM: &str = "System default";

pub struct Panel {
    settings: Entity<Settings>,
    shells: Entity<SelectState<Vec<SharedString>>>,
    notes: Entity<NotesFolders>,
    _subscriptions: Vec<Subscription>,
}

impl Panel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = store(cx).settings.clone();
        let (names, selected) = choices(&settings, cx);
        let shells = cx.new(|cx| SelectState::new(names, selected, window, cx));
        let notes = cx.new(|cx| NotesFolders::new(window, cx));
        let subscriptions = vec![
            cx.observe(&settings, |_, _, cx| cx.notify()),
            cx.subscribe(
                &shells,
                |this, _, event: &SelectEvent<Vec<SharedString>>, cx| {
                    let SelectEvent::Confirm(Some(name)) = event else {
                        return;
                    };
                    let program = this
                        .settings
                        .read(cx)
                        .shells
                        .iter()
                        .flatten()
                        .find(|option| option.name == name.as_ref())
                        .map(|option| option.program.clone());
                    let settings = this.settings.clone();
                    settings.update(cx, |settings, cx| settings.set_shell(Shell { program }, cx));
                },
            ),
        ];
        Self {
            settings,
            shells,
            notes,
            _subscriptions: subscriptions,
        }
    }

    fn switch(
        &self,
        id: &'static str,
        label: impl Into<SharedString>,
        checked: Option<bool>,
        change: impl Fn(&mut Settings, bool, &mut Context<Settings>) + 'static,
    ) -> AnyElement {
        let settings = self.settings.clone();
        row(
            label,
            Switch::new(id)
                .checked(checked.unwrap_or(false))
                .disabled(checked.is_none())
                .on_change(move |next: &bool, _, cx| {
                    let next = *next;
                    settings.update(cx, |settings, cx| change(settings, next, cx));
                }),
        )
        .into_any_element()
    }

    fn appearance(&self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.settings.read(cx).theme.unwrap_or_default();
        let themes = [
            (Theme::System, "System", Glyph::Desktop),
            (Theme::Light, "Light", Glyph::Sun),
            (Theme::Dark, "Dark", Glyph::Moon),
        ];
        let buttons = h_flex().children(themes.into_iter().map(|(choice, label, icon)| {
            let settings = self.settings.clone();
            Button::new(label)
                .outline()
                .small()
                .selected(current == choice)
                .icon(Icon::new(icon))
                .tooltip(label)
                .on_click(move |_, _, cx| {
                    settings.update(cx, |settings, cx| settings.set_theme(choice, cx));
                })
        }));
        let mut appearance = section("Appearance", None, None).child(row("Theme", buttons));
        if theme::can_be_translucent(cx) {
            let material = self.settings.read(cx).material;
            appearance = appearance.child(self.switch(
                "translucent",
                "Translucent",
                material.map(|material| material == Material::Translucent),
                |settings, on, cx| {
                    settings.set_material(
                        if on {
                            Material::Translucent
                        } else {
                            Material::Solid
                        },
                        cx,
                    );
                },
            ));
        }
        appearance.into_any_element()
    }

    fn stats(&self, cx: &mut Context<Self>) -> AnyElement {
        let stats = self.settings.read(cx).stats;
        let rows: [(&'static str, &str, Toggle); 3] = [
            ("behind", "Commits behind the default branch", |stats| {
                &mut stats.behind
            }),
            ("lines", "Lines changed on the branch", |stats| {
                &mut stats.lines
            }),
            ("upstream", "Commits to push and pull", |stats| {
                &mut stats.upstream
            }),
        ];
        let mut tile_bar = section("Tile bar", None, None);
        for (id, label, field) in rows {
            let checked = stats.map(|mut stats| *field(&mut stats));
            tile_bar = tile_bar.child(self.switch(id, label, checked, move |settings, on, cx| {
                if let Some(mut stats) = settings.stats {
                    *field(&mut stats) = on;
                    settings.set_stats(stats, cx);
                }
            }));
        }
        tile_bar.into_any_element()
    }

    fn usage(&self, cx: &mut Context<Self>) -> AnyElement {
        let shown = self.settings.read(cx).shown.clone();
        let mut status = section("Status bar", None, None);
        for agent in Agent::ALL {
            let checked = shown.as_ref().map(|shown| shown.shows(agent));
            status = status.child(self.switch(
                agent.spec().command,
                format!("{} usage left", agent.spec().short),
                checked,
                move |settings, on, cx| {
                    if let Some(mut shown) = settings.shown.clone() {
                        shown.0.insert(agent, on);
                        settings.set_shown(shown, cx);
                    }
                },
            ));
        }
        status.into_any_element()
    }
}

fn choices(
    settings: &Entity<Settings>,
    cx: &gpui_kit::App,
) -> (Vec<SharedString>, Option<IndexPath>) {
    let settings = settings.read(cx);
    let mut names = vec![SharedString::from(SYSTEM)];
    names.extend(
        settings
            .shells
            .iter()
            .flatten()
            .map(|option| SharedString::from(option.name.clone())),
    );
    let current = settings
        .shell
        .as_ref()
        .and_then(|shell| shell.program.clone());
    let index = current
        .and_then(|program| {
            settings
                .shells
                .iter()
                .flatten()
                .position(|option| option.program == program)
        })
        .map_or(0, |index| index + 1);
    (names, Some(IndexPath::new(index)))
}

impl Render for Panel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let reader = self
            .settings
            .read(cx)
            .accessibility
            .map(|accessibility| accessibility.screen_reader);
        let alerts = self.settings.read(cx).notifications;

        div()
            .id("settings")
            .max_h(px(560.))
            .overflow_y_scroll()
            .child(
                v_flex()
                    .gap_6()
                    .pr_2()
                    .child(self.appearance(cx))
                    .child(self.stats(cx))
                    .child(self.usage(cx))
                    .child(
                        section("Terminal", None, None)
                            .child(row("Shell", Select::new(&self.shells).small().w(px(200.)))),
                    )
                    .child(self.notes.clone())
                    .child(
                        section("Alerts", None, None)
                            .child(self.switch(
                                "notifications",
                                "Notifications",
                                alerts.map(|alerts| alerts.enabled),
                                |settings, on, cx| {
                                    if let Some(mut alerts) = settings.notifications {
                                        alerts.enabled = on;
                                        settings.set_notifications(alerts, cx);
                                    }
                                },
                            ))
                            .child(self.switch(
                                "sound",
                                "Sound",
                                alerts.map(|alerts| alerts.sound),
                                |settings, on, cx| {
                                    if let Some(mut alerts) = settings.notifications {
                                        alerts.sound = on;
                                        settings.set_notifications(alerts, cx);
                                    }
                                },
                            )),
                    )
                    .child(section("Accessibility", None, None).child(self.switch(
                        "screen-reader",
                        "Screen reader",
                        reader,
                        |settings, on, cx| {
                            settings.set_accessibility(Accessibility { screen_reader: on }, cx);
                        },
                    )))
                    .child(super::machines::section(cx)),
            )
    }
}
