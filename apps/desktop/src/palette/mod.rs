pub mod entries;

use std::rc::Rc;

use gpui_kit::component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::{IndexPath, WindowExt};
use gpui_kit::{
    App, AppContext, Context, Entity, Global, IntoElement, ParentElement, Render, Styled,
    WeakEntity, Window, px,
};

use crate::notes::palette::{Source, section};

pub type Run = Rc<dyn Fn(&mut Window, &mut App)>;

pub struct Section {
    pub label: String,
    pub items: Vec<(CommandItem, Run)>,
}

pub struct Palette {
    state: Entity<CommandState>,
    before: Vec<Section>,
    after: Vec<Section>,
    notes: Option<Source>,
    query: String,
}

struct Open(WeakEntity<Palette>);

impl Global for Open {}

impl Render for Palette {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notes = self
            .notes
            .as_ref()
            .and_then(|source| section(source, &self.query));
        let sections: Vec<&Section> = self
            .before
            .iter()
            .chain(notes.as_ref())
            .chain(&self.after)
            .collect();
        let runs: Vec<Vec<Run>> = sections
            .iter()
            .map(|section| section.items.iter().map(|(_, run)| run.clone()).collect())
            .collect();
        let me = cx.entity().downgrade();
        let mut command = Command::new(&self.state)
            .placeholder("Go to or run…")
            .bordered(false)
            .max_h(px(420.))
            .empty(|_, _, _| "Nothing matches.")
            .on_query(move |query, _, cx| {
                let query = query.to_owned();
                me.update(cx, |palette, cx| {
                    palette.query = query;
                    cx.notify();
                })
                .ok();
            })
            .on_confirm(move |path: IndexPath, window, cx| {
                let Some(run) = runs
                    .get(path.section)
                    .and_then(|section| section.get(path.row))
                    .cloned()
                else {
                    return;
                };
                window.close_dialog(cx);
                window.defer(cx, move |window, cx| run(window, cx));
            })
            .on_cancel(WindowExt::close_dialog);
        for section in sections {
            let group = CommandGroup::new()
                .label(section.label.clone())
                .items(section.items.iter().map(|(item, _)| item.clone()));
            command = command.group(group);
        }
        command.w_full()
    }
}

pub fn toggle(window: &mut Window, cx: &mut App) {
    let open = cx
        .try_global::<Open>()
        .and_then(|open| open.0.upgrade())
        .is_some();
    if open && window.has_active_dialog(cx) {
        window.close_dialog(cx);
        return;
    }

    let machine = crate::workspaces::desk::current(cx)
        .and_then(|desk| desk.read(cx).view().map(|view| view.machine.clone()));
    if let Some(machine) = machine {
        let notes = crate::state::store(cx).notes.clone();
        notes.update(cx, |notes, cx| notes.ensure(&machine, cx));
    }

    let Some((before, after, notes)) = entries::sections(cx) else {
        return;
    };
    let state = cx.new(|cx| CommandState::new(window, cx));
    let palette = cx.new(|_| Palette {
        state: state.clone(),
        before,
        after,
        notes,
        query: String::new(),
    });
    cx.set_global(Open(palette.downgrade()));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .w(px(560.))
            .close_button(false)
            .child(palette.clone())
    });
    state.update(cx, |state, cx| state.focus(window, cx));
}
