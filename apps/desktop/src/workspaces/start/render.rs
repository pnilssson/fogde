use super::{Launch, StartPage};
use crate::assets::Glyph;
use crate::paths;
use crate::state::machines::Machine;
use crate::state::store;
use crate::theme::Type;
use crate::workspaces::selection::main_checkout;
use crate::workspaces::{agent_icon, dialogs};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use protocol::workspaces::Workspace;

impl StartPage {
    fn choices(&self, machine: &Machine, cx: &mut Context<Self>) -> AnyElement {
        let Some(agents) = store(cx)
            .agents
            .read(cx)
            .list(&machine.id)
            .map(|_| store(cx).agents.read(cx).signed_in(&machine.id))
        else {
            return div()
                .h_8()
                .flex()
                .items_center()
                .child(Spinner::new())
                .into_any_element();
        };
        let busy = self.opening.is_some();
        let mut row = h_flex().flex_wrap().justify_center().gap_2();
        for agent in std::iter::once(None).chain(agents.into_iter().map(Some)) {
            let opening = self.opening == Some(Launch(agent));
            let name = agent.map_or("Terminal", |agent| agent.spec().name);
            let id = agent.map_or("terminal", |agent| agent.spec().command);
            let mut button = Button::new(id)
                .secondary()
                .small()
                .disabled(busy)
                .label(if opening { "Opening…" } else { name })
                .on_click(cx.listener(move |this, _, window, cx| this.open(agent, window, cx)));
            if agent.is_none() {
                button = button.track_focus(&self.first);
            }
            button = match (opening, agent) {
                (true, _) => button.loading(true),
                (false, Some(agent)) => button.child(agent_icon(agent, cx)),
                (false, None) => button.icon(Icon::new(Glyph::TerminalWindow)),
            };
            row = row.child(button);
        }
        row.child(
            Button::new("notes")
                .secondary()
                .small()
                .disabled(busy)
                .icon(Icon::new(Glyph::Notebook))
                .label("Notes")
                .on_click(cx.listener(|this, _, window, cx| this.notes(window, cx))),
        )
        .into_any_element()
    }

    fn worktree(
        &self,
        workspace: &Workspace,
        folder: &str,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        main_checkout(workspace)?;
        let checkout = workspace
            .checkouts
            .iter()
            .find(|checkout| checkout.path == folder)?;
        if !checkout.main {
            return Some(
                Button::new("remove-worktree")
                    .secondary()
                    .small()
                    .loading(self.removing)
                    .label(if self.removing {
                        "Removing…"
                    } else {
                        "Remove worktree"
                    })
                    .on_click(cx.listener(|this, _, window, cx| this.remove_worktree(window, cx)))
                    .into_any_element(),
            );
        }

        Some(
            v_flex()
                .w_full()
                .max_w_80()
                .gap_2()
                .child(div().type_body().child("New worktree"))
                .child(
                    Input::new(&self.name).suffix(
                        Button::new("create-worktree")
                            .ghost()
                            .xsmall()
                            .loading(self.creating)
                            .label(if self.creating {
                                "Creating…"
                            } else {
                                "Create"
                            })
                            .on_click(
                                cx.listener(|this, _, window, cx| this.create_worktree(window, cx)),
                            ),
                    ),
                )
                .when(self.invalid, |form| {
                    form.child(
                        div()
                            .type_caption()
                            .text_color(cx.theme().danger)
                            .child("Enter a name"),
                    )
                })
                .into_any_element(),
        )
    }
}

fn empty(title: impl Into<gpui_kit::SharedString>, description: impl IntoElement) -> gpui_kit::Div {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .p_6()
        .child(
            v_flex()
                .items_center()
                .gap_2()
                .max_w_96()
                .child(div().type_title().child(title.into()))
                .child(div().type_body().text_center().child(description)),
        )
}

impl Render for StartPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let content = match self.context(cx) {
            None => {
                let creating = self
                    .desk
                    .upgrade()
                    .is_some_and(|desk| desk.read(cx).view().is_none());
                let machine = store(cx).machines.read(cx).connected().next().cloned();
                match (creating, machine) {
                    (true, Some(machine)) => empty(
                        "Create a workspace",
                        div().text_color(muted).child(
                            "A workspace is a folder you work in, usually a repo, with its terminals and agents beside it.",
                        ),
                    )
                    .child(
                        Button::new("new-workspace")
                            .track_focus(&self.first)
                            .primary()
                            .small()
                            .label("New workspace")
                            .on_click(move |_, window, cx| dialogs::new_workspace(&machine, window, cx)),
                    )
                    .into_any_element(),
                    _ => div().into_any_element(),
                }
            }
            Some((view, _, workspace, _)) if workspace.missing => {
                let folder = workspace.folder.clone();
                empty(
                    "Folder not found",
                    div()
                        .text_color(muted)
                        .child(format!("{folder} was moved or deleted.")),
                )
                .child(
                    Button::new("delete-workspace")
                        .track_focus(&self.first)
                        .danger()
                        .small()
                        .label("Delete workspace")
                        .on_click(move |_, window, cx| {
                            dialogs::delete_workspace(&view.machine, &workspace, window, cx);
                        }),
                )
                .into_any_element()
            }
            Some((view, machine, workspace, folder)) => {
                let checkout = workspace
                    .checkouts
                    .iter()
                    .find(|checkout| checkout.path == folder);
                let place = match checkout.filter(|checkout| !checkout.main) {
                    Some(checkout) => checkout
                        .branch
                        .clone()
                        .unwrap_or_else(|| paths::name(&checkout.path).to_owned()),
                    None => workspace.name.clone(),
                };
                let spoken = if machine.local() {
                    "this computer".to_owned()
                } else {
                    machine.name.to_string()
                };
                let connected =
                    store(cx).machines.read(cx).state(&view.machine) == client::State::Connected;
                empty(
                    format!("Continue working in {place} on {spoken}"),
                    div().text_color(muted).child(folder.clone()),
                )
                .when(connected, |page| {
                    page.child(self.choices(&machine, cx))
                        .children(self.worktree(&workspace, &folder, cx))
                })
                .into_any_element()
            }
        };

        div()
            .id("start")
            .track_focus(&self.focus)
            .size_full()
            .overflow_y_scroll()
            .child(content)
    }
}
