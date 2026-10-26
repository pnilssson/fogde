use client::Client;
use gpui_kit::{App, Context, Task};
use protocol::Method;
use protocol::accessibility::{Accessibility, GetAccessibility, SetAccessibility};
use protocol::notifications::{GetNotifications, Notifications, SetNotifications};
use protocol::shell::{GetShell, ListShells, SetShell, Shell, ShellOption};
use protocol::stats::{GetStats, SetStats, Stats};
use protocol::theme::{GetMaterial, GetTheme, Material, SetMaterial, SetTheme, Theme};
use protocol::usage::{GetShown, SetShown, Shown};

use crate::notice;
use crate::runtime;

#[derive(Default)]
pub struct Settings {
    own: Option<Client>,
    pub theme: Option<Theme>,
    pub material: Option<Material>,
    pub stats: Option<Stats>,
    pub notifications: Option<Notifications>,
    pub accessibility: Option<Accessibility>,
    pub shown: Option<Shown>,
    pub shell: Option<Shell>,
    pub shells: Option<Vec<ShellOption>>,
    loading: Vec<Task<()>>,
}

type Field<T> = fn(&mut Settings) -> &mut Option<T>;

impl Settings {
    pub fn new(own: Client, cx: &mut Context<Self>) -> Self {
        let mut settings = Self {
            own: Some(own),
            ..Self::default()
        };
        settings.load::<GetTheme>(|s| &mut s.theme, cx);
        settings.load::<GetMaterial>(|s| &mut s.material, cx);
        settings.load::<GetStats>(|s| &mut s.stats, cx);
        settings.load::<GetNotifications>(|s| &mut s.notifications, cx);
        settings.load::<GetAccessibility>(|s| &mut s.accessibility, cx);
        settings.load::<GetShown>(|s| &mut s.shown, cx);
        settings.load::<GetShell>(|s| &mut s.shell, cx);
        settings.load::<ListShells>(|s| &mut s.shells, cx);
        settings
    }

    fn load<M>(&mut self, field: Field<M::Result>, cx: &mut Context<Self>)
    where
        M: Method<Params = ()>,
        M::Result: Send,
    {
        let Some(own) = &self.own else {
            return;
        };
        let request = runtime::call::<M>(own, (), cx);
        self.loading.push(cx.spawn(async move |this, cx| {
            let result = request.await;
            this.update(cx, |this, cx| match result {
                Ok(value) => {
                    *field(this) = Some(value);
                    cx.notify();
                }
                Err(error) => tracing::warn!(%error, method = M::NAME, "could not load a setting"),
            })
            .ok();
        }));
    }

    fn save<M>(
        &mut self,
        field: Field<M::Params>,
        next: M::Params,
        failure: &'static str,
        cx: &mut Context<Self>,
    ) where
        M: Method<Result = ()>,
        M::Params: Clone + Send + Sync,
    {
        let Some(own) = self.own.clone() else {
            return;
        };
        let before = field(self).replace(next.clone());
        cx.notify();

        let request = runtime::call::<M>(&own, next, cx);
        cx.spawn(async move |this, cx| {
            if let Err(error) = request.await {
                this.update(cx, |this, cx| {
                    *field(this) = before;
                    notice::error(failure, &error, cx);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.save::<SetTheme>(|s| &mut s.theme, theme, "Could not save the theme", cx);
    }

    pub fn set_material(&mut self, material: Material, cx: &mut Context<Self>) {
        self.save::<SetMaterial>(
            |s| &mut s.material,
            material,
            "Could not save the window style",
            cx,
        );
    }

    pub fn set_stats(&mut self, stats: Stats, cx: &mut Context<Self>) {
        self.save::<SetStats>(
            |s| &mut s.stats,
            stats,
            "Could not save the tile bar setting",
            cx,
        );
    }

    pub fn set_notifications(&mut self, notifications: Notifications, cx: &mut Context<Self>) {
        self.save::<SetNotifications>(
            |s| &mut s.notifications,
            notifications,
            "Could not save the alert setting",
            cx,
        );
    }

    pub fn set_accessibility(&mut self, accessibility: Accessibility, cx: &mut Context<Self>) {
        self.save::<SetAccessibility>(
            |s| &mut s.accessibility,
            accessibility,
            "Could not save the accessibility setting",
            cx,
        );
    }

    pub fn set_shown(&mut self, shown: Shown, cx: &mut Context<Self>) {
        self.save::<SetShown>(
            |s| &mut s.shown,
            shown,
            "Could not save the status bar setting",
            cx,
        );
    }

    pub fn set_shell(&mut self, shell: Shell, cx: &mut Context<Self>) {
        self.save::<SetShell>(|s| &mut s.shell, shell, "Could not save the shell", cx);
    }

    pub fn screen_reader(&self) -> bool {
        self.accessibility
            .is_some_and(|accessibility| accessibility.screen_reader)
    }
}

pub fn stats(cx: &App) -> Option<Stats> {
    super::store(cx).settings.read(cx).stats
}
