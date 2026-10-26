use std::time::{Duration, Instant};

use super::doc::Doc;

const GROUP: Duration = Duration::from_secs(1);
const LIMIT: usize = 200;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Typing,
    Other,
}

#[derive(Default)]
pub struct History {
    undo: Vec<Doc>,
    redo: Vec<Doc>,
    last: Option<(Kind, Instant)>,
}

impl History {
    pub fn record(&mut self, before: &Doc, kind: Kind, now: Instant) {
        let grouped = kind == Kind::Typing
            && self
                .last
                .is_some_and(|(last, at)| last == Kind::Typing && now.duration_since(at) < GROUP);
        if !grouped {
            self.undo.push(before.clone());
            if self.undo.len() > LIMIT {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.last = Some((kind, now));
    }

    pub fn undo(&mut self, current: &Doc) -> Option<Doc> {
        let previous = self.undo.pop()?;
        self.redo.push(current.clone());
        self.last = None;
        Some(previous)
    }

    pub fn redo(&mut self, current: &Doc) -> Option<Doc> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        self.last = None;
        Some(next)
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Doc {
        Doc::new(text.to_owned())
    }

    #[test]
    fn groups_quick_typing_into_one_step() {
        let mut history = History::default();
        let start = Instant::now();
        history.record(&doc(""), Kind::Typing, start);
        history.record(&doc("a"), Kind::Typing, start + Duration::from_millis(200));
        history.record(&doc("ab"), Kind::Other, start + Duration::from_millis(400));

        assert_eq!(
            history.undo(&doc("ab\n")).map(|doc| doc.text),
            Some("ab".to_owned())
        );
        assert_eq!(
            history.undo(&doc("ab")).map(|doc| doc.text),
            Some(String::new())
        );
        assert_eq!(history.undo(&doc("")), None);
        assert_eq!(
            history.redo(&doc("")).map(|doc| doc.text),
            Some("ab".to_owned())
        );
    }

    #[test]
    fn a_pause_starts_a_new_step_and_an_edit_drops_redo() {
        let mut history = History::default();
        let start = Instant::now();
        history.record(&doc(""), Kind::Typing, start);
        history.record(&doc("a"), Kind::Typing, start + Duration::from_secs(2));
        assert_eq!(
            history.undo(&doc("ab")).map(|doc| doc.text),
            Some("a".to_owned())
        );

        history.record(&doc("a"), Kind::Other, start + Duration::from_secs(3));
        assert_eq!(history.redo(&doc("ax")), None);
    }
}
