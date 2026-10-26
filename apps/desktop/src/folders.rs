mod render;

use std::rc::Rc;

use client::Client;
use gpui_kit::component::WindowExt;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, SharedString, Subscription, Task, WeakEntity,
    Window, px,
};
use protocol::folders::{CreateFolder, CreateFolderParams, Folders, ListFolders, RemoveFolder};

use crate::notice;
use crate::runtime;

type Choose = Rc<dyn Fn(String, WeakEntity<FolderBrowser>, &mut Window, &mut App)>;

pub struct FolderBrowser {
    client: Client,
    action: SharedString,
    folders: Option<Folders>,
    error: Option<String>,
    path: Entity<InputState>,
    name: Entity<InputState>,
    naming: bool,
    invalid: bool,
    busy: bool,
    on_choose: Choose,
    loading: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl FolderBrowser {
    pub fn new(
        client: Client,
        start: String,
        action: impl Into<SharedString>,
        on_choose: Choose,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let path = cx.new(|cx| InputState::new(window, cx).default_value(start.clone()));
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("New folder"));
        let subscriptions = vec![
            cx.subscribe_in(&path, window, |this, input, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let typed = input.read(cx).value().trim().to_owned();
                    if !typed.is_empty() {
                        this.open(typed, window, cx);
                    }
                }
            }),
            cx.subscribe_in(&name, window, |this, _, event, window, cx| match event {
                InputEvent::PressEnter { .. } => this.create(window, cx),
                InputEvent::Change => {
                    this.invalid = false;
                    cx.notify();
                }
                _ => {}
            }),
        ];
        let mut browser = Self {
            client,
            action: action.into(),
            folders: None,
            error: None,
            path,
            name,
            naming: false,
            invalid: false,
            busy: false,
            on_choose,
            loading: None,
            _subscriptions: subscriptions,
        };
        browser.open(start, window, cx);
        browser
    }

    pub fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        self.busy = busy;
        cx.notify();
    }

    fn open(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let request = runtime::call::<ListFolders>(&self.client, path, cx);
        self.loading = Some(cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| {
                this.loading = None;
                match result {
                    Ok(folders) => {
                        let shown = folders.path.clone();
                        this.path
                            .update(cx, |path, cx| path.set_value(shown, window, cx));
                        this.folders = Some(folders);
                        this.error = None;
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.folders.as_ref().map(|folders| folders.path.clone()) {
            self.open(path, window, cx);
        }
    }

    fn create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.invalid = true;
            cx.notify();
            return;
        }
        let Some(parent) = self.folders.as_ref().map(|folders| folders.path.clone()) else {
            return;
        };
        let request =
            runtime::call::<CreateFolder>(&self.client, CreateFolderParams { parent, name }, cx);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| match result {
                Ok(folder) => {
                    this.naming = false;
                    this.name
                        .update(cx, |name, cx| name.set_value("", window, cx));
                    this.open(folder.path, window, cx);
                }
                Err(error) => notice::error("Could not create the folder", &error, cx),
            })
            .ok();
        })
        .detach();
    }

    fn remove(&mut self, path: String, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let request = runtime::call::<RemoveFolder>(&self.client, path, cx);
        cx.spawn_in(window, async move |this, cx| {
            let result = request.await;
            this.update_in(cx, |this, window, cx| match result {
                Ok(()) => this.reload(window, cx),
                Err(error) => notice::error(format!("Could not delete {name}"), &error, cx),
            })
            .ok();
        })
        .detach();
    }
}

pub fn open(
    client: Client,
    start: String,
    title: &'static str,
    action: &'static str,
    on_choose: Choose,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FolderBrowser> {
    let browser = cx.new(|cx| FolderBrowser::new(client, start, action, on_choose, window, cx));
    let shown = browser.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title).w(px(480.)).child(shown.clone())
    });
    browser
}
