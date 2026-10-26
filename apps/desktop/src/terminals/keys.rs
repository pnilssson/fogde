use gpui_kit::Keystroke;
use protocol::terminals::Key as Named;
use terminal::{Key, Modifiers};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Press {
    Bytes(Vec<u8>),
    Named(Named),
    Copy,
    Paste,
    Text,
    App,
}

fn named(key: &str) -> Option<Key> {
    Some(match key {
        "enter" => Key::Enter,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "escape" => Key::Escape,
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "insert" => Key::Insert,
        "delete" => Key::Delete,
        "space" => Key::Char(' '),
        _ => {
            let number = key.strip_prefix('f')?.parse::<u8>().ok()?;
            Key::F(number)
        }
    })
}

pub fn press(keystroke: &Keystroke, macos: bool, selection: bool) -> Press {
    let modifiers = keystroke.modifiers;
    if modifiers.platform {
        return Press::App;
    }

    let key = keystroke.key.as_str();
    let only_shift = modifiers.shift && !modifiers.control && !modifiers.alt;
    let only_control = modifiers.control && !modifiers.shift && !modifiers.alt;
    if key == "enter" && only_shift {
        return Press::Named(Named::ShiftEnter);
    }
    if key == "enter" && only_control {
        return Press::Named(Named::CtrlEnter);
    }

    if !macos && modifiers.control && !modifiers.alt {
        if key == "v" {
            return Press::Paste;
        }
        if key == "c" && !modifiers.shift && selection {
            return Press::Copy;
        }
    }

    let held = Modifiers {
        shift: modifiers.shift,
        alt: modifiers.alt,
        control: modifiers.control,
    };
    if let Some(named) = named(key) {
        if named == Key::Char(' ') && !modifiers.control && !modifiers.alt {
            return Press::Text;
        }
        return terminal::encode(named, held).map_or(Press::App, Press::Bytes);
    }

    if !modifiers.control && !modifiers.alt {
        return Press::Text;
    }

    let mut chars = key.chars();
    let (Some(char), None) = (chars.next(), chars.next()) else {
        return Press::App;
    };
    let char = if modifiers.alt && !modifiers.control {
        keystroke
            .key_char
            .as_deref()
            .and_then(|text| text.chars().next())
            .unwrap_or(char)
    } else {
        char
    };

    terminal::encode(Key::Char(char), held).map_or(Press::App, Press::Bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &'static str) -> Keystroke {
        Keystroke::parse(text).unwrap()
    }

    #[test]
    fn named_chords_go_to_the_host() {
        assert_eq!(
            press(&key("shift-enter"), false, false),
            Press::Named(Named::ShiftEnter)
        );
        assert_eq!(
            press(&key("ctrl-enter"), true, false),
            Press::Named(Named::CtrlEnter)
        );
    }

    #[test]
    fn clipboard_chords_work_outside_macos_only() {
        assert_eq!(press(&key("ctrl-v"), false, false), Press::Paste);
        assert_eq!(press(&key("ctrl-shift-v"), false, false), Press::Paste);
        assert_eq!(press(&key("ctrl-c"), false, true), Press::Copy);
        assert_eq!(press(&key("ctrl-c"), false, false), Press::Bytes(vec![3]));
        assert_eq!(press(&key("ctrl-c"), true, true), Press::Bytes(vec![3]));
    }

    #[test]
    fn plain_typing_goes_through_text_input() {
        assert_eq!(press(&key("a"), false, false), Press::Text);
        assert_eq!(press(&key("shift-a"), false, false), Press::Text);
        assert_eq!(press(&key("space"), false, false), Press::Text);
    }

    #[test]
    fn special_keys_and_modifiers_encode_like_xterm() {
        assert_eq!(
            press(&key("up"), false, false),
            Press::Bytes(b"\x1b[A".to_vec())
        );
        assert_eq!(
            press(&key("backspace"), false, false),
            Press::Bytes(b"\x7f".to_vec())
        );
        assert_eq!(
            press(&key("shift-tab"), false, false),
            Press::Bytes(b"\x1b[Z".to_vec())
        );
        assert_eq!(
            press(&key("alt-b"), false, false),
            Press::Bytes(b"\x1bb".to_vec())
        );
        assert_eq!(
            press(&key("ctrl-space"), false, false),
            Press::Bytes(vec![0])
        );
    }

    #[test]
    fn command_chords_stay_with_the_app() {
        assert_eq!(press(&key("cmd-k"), true, false), Press::App);
    }
}
