use protocol::notes::Written;

use super::{Loaded, Outcome, Session};

struct File {
    text: String,
    version: u32,
    fail: bool,
    writes: u32,
}

impl File {
    fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            version: 1,
            fail: false,
            writes: 0,
        }
    }

    fn write(&mut self, text: &str, version: &str) -> Result<Written, ()> {
        self.writes += 1;
        if self.fail {
            return Err(());
        }
        if version != self.version.to_string() {
            return Ok(Written::Changed);
        }
        self.text = text.into();
        self.version += 1;
        Ok(Written::Saved {
            version: self.version.to_string(),
        })
    }

    fn agent(&mut self, text: &str) {
        self.text = text.into();
        self.version += 1;
    }
}

struct Editor {
    shown: String,
    session: Session,
    pending: bool,
    kept: u32,
}

impl Editor {
    fn open(file: &File) -> Self {
        Self {
            shown: file.text.clone(),
            session: Session::new(file.text.clone(), file.version.to_string()),
            pending: false,
            kept: 0,
        }
    }

    fn kind(&mut self, text: &str) {
        self.shown = text.into();
        self.pending = true;
    }

    fn loaded(&mut self, text: &str, version: &str) {
        if let Loaded::Applied {
            replace,
            kept,
            save,
        } = self
            .session
            .loaded(text.into(), version.into(), &self.shown)
        {
            if let Some(text) = replace {
                self.shown = text;
            }
            self.kept += u32::from(kept);
            self.pending |= save;
        }
    }

    fn run(&mut self, file: &mut File) {
        while std::mem::take(&mut self.pending) {
            let Some((text, version)) = self.session.begin(&self.shown) else {
                continue;
            };
            let result = file.write(&text, &version);
            let (saved, waiting) = self.session.finish(text, result, &self.shown.clone());
            if let Some((text, version)) = waiting {
                self.loaded(&text, &version);
                continue;
            }
            match saved {
                Outcome::Saved { again, .. } => self.pending |= again,
                Outcome::Changed => {
                    let (text, version) = (file.text.clone(), file.version.to_string());
                    self.loaded(&text, &version);
                }
                Outcome::Failed => {}
            }
        }
    }
}

#[test]
fn saves_typing_after_a_pause() {
    let mut file = File::new("# Plan\n");
    let mut editor = Editor::open(&file);

    editor.kind("# Plan\n\n- login\n");
    assert_eq!(file.text, "# Plan\n");
    editor.run(&mut file);

    assert_eq!(file.text, "# Plan\n\n- login\n");
}

#[test]
fn shows_an_agents_change_when_nothing_is_unsaved() {
    let mut file = File::new("# Plan\n");
    let mut editor = Editor::open(&file);

    file.agent("# Plan\n\n- signup\n");
    editor.loaded(&file.text.clone(), &file.version.to_string());

    assert_eq!(editor.shown, "# Plan\n\n- signup\n");
}

#[test]
fn merges_an_agents_change_into_unsaved_typing_and_saves_both() {
    let mut file = File::new("# Plan\n\n- login\n");
    let mut editor = Editor::open(&file);

    editor.kind("# Plan for login\n\n- login\n");
    file.agent("# Plan\n\n- login\n- signup\n");
    editor.run(&mut file);

    assert_eq!(file.text, "# Plan for login\n\n- login\n- signup\n");
    assert_eq!(editor.shown, file.text);
    assert_eq!(editor.kept, 0);
}

#[test]
fn a_late_reload_of_its_own_earlier_save_leaves_newer_typing_in_place() {
    let mut file = File::new("# Plan\n");
    let mut editor = Editor::open(&file);

    editor.kind("# Plan\n\n- login\n");
    editor.run(&mut file);
    let earlier = (file.text.clone(), file.version.to_string());
    editor.kind("# Plan\n\n- login\n- signup\n");
    editor.run(&mut file);

    editor.loaded(&earlier.0, &earlier.1);

    assert_eq!(editor.shown, "# Plan\n\n- login\n- signup\n");
    assert_eq!(file.text, "# Plan\n\n- login\n- signup\n");
}

#[test]
fn keeps_the_users_text_when_an_agent_changed_the_same_line() {
    let mut file = File::new("# Plan\n\n- login\n");
    let mut editor = Editor::open(&file);

    editor.kind("# Plan\n\n- login with email\n");
    file.agent("# Plan\n\n- login with SSO\n");
    editor.run(&mut file);

    assert_eq!(file.text, "# Plan\n\n- login with email\n");
    assert_eq!(editor.kept, 1);
}

#[test]
fn a_failed_save_waits_for_the_next_edit_instead_of_retrying() {
    let mut file = File::new("# Plan\n");
    file.fail = true;
    let mut editor = Editor::open(&file);

    editor.kind("# Plan\n\n- login\n");
    editor.run(&mut file);
    assert_eq!(file.writes, 1);

    editor.kind("# Plan\n\n- login\n- signup\n");
    editor.run(&mut file);
    assert_eq!(file.writes, 2);
}

#[test]
fn never_writes_a_note_that_was_only_opened() {
    let mut file = File::new("# Plan\r\n");
    let mut editor = Editor::open(&file);

    editor.pending = true;
    editor.run(&mut file);

    assert_eq!(file.version, 1);
    assert_eq!(file.writes, 0);
}
