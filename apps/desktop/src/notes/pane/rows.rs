use super::{create, menu, order, set_order};
use crate::assets::Glyph;
use crate::notes::tree::{Folder, Order, build};
use crate::notes::view::NotesTile;
use crate::state::notes::Kind;
use crate::state::store;
use crate::theme::{self, Type};
use crate::workspaces::desk::split::Blank;
use crate::workspaces::tiles::tile::Dragged;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{ContextMenuExt, DropdownMenu, PopupMenuItem};
use gpui_kit::component::{ActiveTheme, Icon, Sizable, h_flex, v_flex};
use gpui_kit::{
    AnyElement, AppContext, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};
use protocol::notes::Note;

impl NotesTile {
    fn note_row(&self, note: &Note, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let active = self.note.as_deref() == Some(note.path.as_str());
        let tokens = theme::tokens(cx);
        let (hover, selected) = (tokens.hover, tokens.selected);
        let path = note.path.clone();
        let tile = cx.entity();
        let (machine, menu_path, name) =
            (self.machine.clone(), note.path.clone(), note.name.clone());
        let dragged = Dragged::Note {
            machine: self.machine.clone(),
            path: note.path.clone(),
        };
        h_flex()
            .id(SharedString::from(format!("note-{}", note.path)))
            .h_7()
            .gap_2()
            .pl(px(8. + 12. * crate::num::real_usize(depth)))
            .pr_2()
            .rounded_md()
            .type_body()
            .cursor_pointer()
            .when(active, |row| row.bg(selected))
            .when(!active, |row| row.hover(|style| style.bg(hover)))
            .child(Icon::new(Glyph::FileText).small())
            .child(div().truncate().child(note.name.clone()))
            .on_click(cx.listener(move |this, _, window, cx| this.open(&path, window, cx)))
            .on_drag(dragged, |_, _, _, cx| cx.new(|_| Blank))
            .context_menu(move |popup, _, _| menu(popup, &tile, &machine, &menu_path, &name, false))
            .into_any_element()
    }

    fn folder_rows(
        &self,
        folder: &Folder,
        depth: usize,
        rows: &mut Vec<AnyElement>,
        cx: &mut Context<Self>,
    ) {
        for inner in &folder.folders {
            let open = self
                .expanded
                .contains(inner.path.trim_end_matches(['/', '\\']));
            let path = inner.path.trim_end_matches(['/', '\\']).to_owned();
            let hover = theme::tokens(cx).hover;
            let tile = cx.entity();
            let (machine, menu_path, name) =
                (self.machine.clone(), inner.path.clone(), inner.name.clone());
            rows.push(
                h_flex()
                    .id(SharedString::from(format!("folder-{}", inner.path)))
                    .h_7()
                    .gap_2()
                    .pl(px(8. + 12. * crate::num::real_usize(depth)))
                    .pr_2()
                    .rounded_md()
                    .type_body()
                    .cursor_pointer()
                    .hover(|style| style.bg(hover))
                    .child(
                        Icon::new(if open {
                            Glyph::CaretDown
                        } else {
                            Glyph::CaretRight
                        })
                        .small(),
                    )
                    .child(div().truncate().child(inner.name.clone()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.expanded.remove(&path) {
                            this.expanded.insert(path.clone());
                        }
                        cx.notify();
                    }))
                    .context_menu(move |popup, _, _| {
                        menu(popup, &tile, &machine, &menu_path, &name, true)
                    })
                    .into_any_element(),
            );
            if open {
                self.folder_rows(inner, depth + 1, rows, cx);
            }
        }
        for note in &folder.notes {
            rows.push(self.note_row(note, depth, cx));
        }
    }

    pub(crate) fn pane(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let Some(list) = store(cx).notes.read(cx).list(&self.machine).cloned() else {
            return div().into_any_element();
        };
        let current = order(cx);
        let tree = build(&list, current);
        let parent = self
            .note
            .as_ref()
            .and_then(|note| list.notes.iter().find(|known| known.path == *note))
            .map_or_else(|| list.root.clone(), |note| note.folder.clone());
        let mut rows = Vec::new();
        self.folder_rows(&tree, 0, &mut rows, cx);
        let tile = cx.entity();
        let (note_tile, folder_tile) = (tile.clone(), tile.clone());
        let (note_parent, folder_parent) = (parent.clone(), parent);

        v_flex()
            .w_64()
            .flex_shrink_0()
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h_8()
                    .flex_shrink_0()
                    .gap_1()
                    .px_2()
                    .child(
                        Button::new("new-note")
                            .ghost()
                            .small()
                            .icon(Icon::new(Glyph::FilePlus))
                            .tooltip("New note")
                            .on_click(move |_, window, cx| {
                                create(&note_tile, Kind::Note, note_parent.clone(), window, cx);
                            }),
                    )
                    .child(
                        Button::new("new-folder")
                            .ghost()
                            .small()
                            .icon(Icon::new(Glyph::FolderPlus))
                            .tooltip("New folder")
                            .on_click(move |_, window, cx| {
                                create(
                                    &folder_tile,
                                    Kind::Folder,
                                    folder_parent.clone(),
                                    window,
                                    cx,
                                );
                            }),
                    )
                    .child(
                        Button::new("sort")
                            .ghost()
                            .small()
                            .icon(Icon::new(Glyph::SortAscending))
                            .tooltip("Sort notes")
                            .dropdown_menu(move |popup, _, _| {
                                Order::ALL.into_iter().fold(popup, |popup, order| {
                                    popup.item(
                                        PopupMenuItem::new(order.label())
                                            .checked(order == current)
                                            .on_click(move |_, _, cx| set_order(order, cx)),
                                    )
                                })
                            }),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("hide-notes")
                            .ghost()
                            .small()
                            .icon(Icon::new(Glyph::SidebarSimpleFill))
                            .tooltip("Hide notes list")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.browsing = false;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                v_flex()
                    .id("notes-tree")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_1()
                    .children(rows),
            )
            .into_any_element()
    }
}
