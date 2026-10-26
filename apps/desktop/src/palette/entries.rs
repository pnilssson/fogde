mod parts;

use parts::{lists, tile_section, worktree_section};

use std::rc::Rc;

use gpui_kit::component::command::CommandItem;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{Disableable, Icon, IconNamed, Sizable, h_flex};
use gpui_kit::{App, Entity, ParentElement, Styled, Window};
use protocol::agents::Agent;
use protocol::terminals::CreateTerminalParams;

use super::{Run, Section};
use crate::assets::Glyph;
use crate::keymap::{NewTile, NextWaiting};
use crate::state::machines::Machine;
use crate::state::store;
use crate::workspaces::desk::{self, Desk};
use crate::workspaces::selection::{Group, Waiting, folder_of, main_checkout, waiting};
use crate::workspaces::tiles::tile::{Selection, View};

pub fn item(label: impl Into<String>, icon: impl IconNamed, keywords: &[&str]) -> CommandItem {
    CommandItem::new()
        .label(label.into())
        .icon(Icon::new(icon))
        .keywords(keywords.iter().map(|keyword| (*keyword).to_owned()))
}

fn dispatch(action: impl gpui_kit::Action + Clone) -> Run {
    Rc::new(move |window, cx| window.dispatch_action(action.boxed_clone(), cx))
}

fn with_desk(run: impl Fn(&mut Desk, &mut Window, &mut gpui_kit::Context<Desk>) + 'static) -> Run {
    let run = Rc::new(run);
    Rc::new(move |window, cx| {
        if let Some(desk) = desk::current(cx) {
            let run = run.clone();
            desk.update(cx, |desk, cx| run(desk, window, cx));
        }
    })
}

fn reveal(view: View, terminal: Option<Selection>) -> Run {
    with_desk(move |desk, window, cx| {
        desk.select_and_show(view.clone(), terminal.clone(), window, cx);
    })
}

pub fn sections(
    cx: &mut App,
) -> Option<(
    Vec<Section>,
    Vec<Section>,
    Option<crate::notes::palette::Source>,
)> {
    let desk = desk::current(cx)?;
    let state = store(cx);
    let machines: Vec<Machine> = state.machines.read(cx).connected().cloned().collect();
    let workspaces = state.workspaces.read(cx);
    let groups: Vec<Group> = machines
        .iter()
        .map(|machine| Group {
            machine: &machine.id,
            workspaces: workspaces.cache(&machine.id).and_then(|cache| cache.list()),
        })
        .collect();
    let needs = waiting(&groups);

    let mut before: Vec<Section> = needs_you(&needs).into_iter().collect();
    before.extend(workspace_sections(&desk, &machines, cx));
    let mut after = lists(&machines, &groups, &needs);
    after.extend(tile_section(&desk, cx));
    Some((before, after, crate::notes::palette::source(&desk, cx)))
}

fn needs_you(needs: &[Waiting]) -> Option<Section> {
    if needs.is_empty() {
        return None;
    }
    let mut items = vec![(
        item(
            "Go to the next agent that needs you",
            Glyph::Bell,
            &["next", "attention"],
        ),
        dispatch(NextWaiting),
    )];
    for waiting in needs {
        items.push((
            item(
                format!("{}  {}", waiting.name, waiting.place),
                Glyph::TerminalWindow,
                &[&waiting.name, &waiting.place],
            ),
            reveal(waiting.view.clone(), Some(waiting.terminal.clone())),
        ));
    }
    Some(Section {
        label: "Needs you".into(),
        items,
    })
}

fn create(machine: String, params: CreateTerminalParams) -> Run {
    Rc::new(move |window, cx| {
        let (machine, params) = (machine.clone(), params.clone());
        let task = store(cx).workspaces.clone().update(cx, |workspaces, cx| {
            workspaces.create_terminal(&machine, params, cx)
        });
        window
            .spawn(cx, async move |cx| {
                let Some(terminal) = task.await else {
                    return;
                };
                cx.update(|window, cx| {
                    if let Some(desk) = desk::current(cx) {
                        let selection = Selection::new(machine, terminal.id);
                        desk.update(cx, |desk, cx| {
                            desk.edit(window, cx, |board| board.opened(selection));
                        });
                    }
                })
                .ok();
            })
            .detach();
    })
}

fn workspace_sections(desk: &Entity<Desk>, machines: &[Machine], cx: &App) -> Vec<Section> {
    let state = store(cx);
    let board = desk.read(cx);
    let (Some(view), Some(workspace)) = (board.view().cloned(), board.workspace(cx).cloned())
    else {
        return Vec::new();
    };
    if workspace.missing || state.machines.read(cx).state(&view.machine) != client::State::Connected
    {
        return Vec::new();
    }

    let folder = folder_of(&workspace, &view);
    let notes_machine = view.machine.clone();
    let mut items = vec![
        (
            item("New tile", Glyph::PlusCircle, &["new", "open", "start"]),
            dispatch(NewTile),
        ),
        (
            item("Open notes", Glyph::Notebook, &["view", "show", "tile"]),
            with_desk(move |desk, window, cx| {
                let machine = notes_machine.clone();
                desk.edit(window, cx, |board| {
                    board.open_notes(&machine, None, crate::notes::key());
                });
            }),
        ),
    ];
    let loading = state.agents.read(cx).list(&view.machine).is_none();
    let agents = std::iter::once(None).chain(
        state
            .agents
            .read(cx)
            .signed_in(&view.machine)
            .into_iter()
            .map(Some),
    );
    for agent in agents {
        let name = agent.map_or("terminal", |agent| agent.spec().name);
        let params = CreateTerminalParams {
            workspace: workspace.id.clone(),
            folder: folder.clone(),
            agent,
        };
        items.push((
            item(
                format!("New {name}"),
                Glyph::TerminalWindow,
                &["new", "open", "start"],
            ),
            create(view.machine.clone(), params),
        ));
    }
    if loading {
        items.push((loading_agents(), Rc::new(|_: &mut Window, _: &mut App| {})));
    }
    if main_checkout(&workspace).is_some()
        && let Some(machine) = machines
            .iter()
            .find(|machine| machine.id == view.machine)
            .cloned()
    {
        let found = workspace.clone();
        items.push((
            item(
                "New worktree",
                Glyph::GitBranch,
                &["new", "create", "branch"],
            ),
            Rc::new(move |window, cx| crate::sidebar::new_worktree(&machine, &found, window, cx)),
        ));
    }

    let mut sections = vec![Section {
        label: format!("New in {}", workspace.name),
        items,
    }];
    sections.extend(worktree_section(&view, &workspace, &folder));
    sections
}

fn loading_agents() -> CommandItem {
    let mut keywords = vec!["new".to_owned(), "open".to_owned(), "start".to_owned()];
    for agent in Agent::ALL {
        keywords.push(agent.spec().name.to_owned());
        keywords.push(agent.spec().short.to_owned());
    }
    CommandItem::new()
        .label("Loading agents…")
        .keywords(keywords)
        .disabled(true)
        .child(|_, _| {
            h_flex()
                .gap_2()
                .child(Spinner::new().small())
                .child("Loading agents…")
        })
}
