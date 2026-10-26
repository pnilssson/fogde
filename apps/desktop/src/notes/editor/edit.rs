use gpui_kit::{Context, Window};

use super::{NoteEditor, display};
use crate::notes::merge::diff;
use crate::notes::session::Loaded;
use crate::notice;

impl NoteEditor {
    pub(super) fn apply(
        &mut self,
        text: String,
        version: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mine = self.disk(cx);
        let Loaded::Applied {
            replace,
            kept,
            save,
        } = self.session.loaded(text, version, &mine)
        else {
            return;
        };
        if let Some(next) = replace {
            self.replace(&display(&next), cx);
        }
        if kept {
            notice::warning(
                "Kept your version of the note",
                Some("The note changed on disk where you were editing, and your edit replaced that change.".into()),
                cx,
            );
        }
        if save {
            self.schedule(window, cx);
        }
    }

    fn replace(&mut self, next: &str, cx: &mut Context<Self>) {
        let current = self.state.read(cx).text().to_owned();
        let Some(change) = diff(&current, next) else {
            return;
        };
        self.state.update(cx, |live, cx| {
            live.apply(change.from..change.to, &change.insert, cx);
        });
    }

    pub(super) fn wrap(&mut self, mark: &str, cx: &mut Context<Self>) {
        self.state.update(cx, |live, cx| live.wrap(mark, cx));
    }

    pub(super) fn click(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let live = self.state.read(cx);
        let (offset, text) = (live.cursor(), live.text().to_owned());
        let follow = self.follow.clone();
        follow(offset, &text, window, cx);
    }
}
