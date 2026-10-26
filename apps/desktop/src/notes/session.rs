#[cfg(test)]
mod tests;

use std::collections::HashSet;

use protocol::notes::Written;

use super::merge::merge;

pub const PAUSE: std::time::Duration = std::time::Duration::from_millis(600);

#[derive(Debug, PartialEq, Eq)]
pub enum Loaded {
    Deferred,
    Ignored,
    Applied {
        replace: Option<String>,
        kept: bool,
        save: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Saved { version: String, again: bool },
    Changed,
    Failed,
}

pub struct Session {
    base: String,
    version: String,
    saving: bool,
    waiting: Option<(String, String)>,
    ours: HashSet<String>,
}

impl Session {
    pub fn new(base: String, version: String) -> Self {
        Self {
            base,
            version,
            saving: false,
            waiting: None,
            ours: HashSet::new(),
        }
    }

    pub fn loaded(&mut self, text: String, version: String, mine: &str) -> Loaded {
        if self.saving {
            self.waiting = Some((text, version));
            return Loaded::Deferred;
        }
        if version == self.version || self.ours.contains(&version) {
            return Loaded::Ignored;
        }

        self.ours.clear();
        let merged = merge(&self.base, mine, &text);
        let kept = merged.is_none();
        let shown = merged.unwrap_or_else(|| mine.to_owned());
        let replace = (shown != mine).then(|| shown.clone());
        self.base = text;
        self.version = version;
        Loaded::Applied {
            save: shown != self.base,
            replace,
            kept,
        }
    }

    pub fn begin(&mut self, text: &str) -> Option<(String, String)> {
        if self.saving || text == self.base {
            return None;
        }
        self.saving = true;
        Some((text.to_owned(), self.version.clone()))
    }

    pub fn finish(
        &mut self,
        text: String,
        result: Result<Written, ()>,
        now: &str,
    ) -> (Outcome, Option<(String, String)>) {
        self.saving = false;
        let saved = match result {
            Ok(Written::Saved { version }) => {
                self.base = text;
                self.version.clone_from(&version);
                self.ours.insert(version.clone());
                Outcome::Saved {
                    again: now != self.base,
                    version,
                }
            }
            Ok(Written::Changed) => Outcome::Changed,
            Err(()) => Outcome::Failed,
        };
        (saved, self.waiting.take())
    }
}
