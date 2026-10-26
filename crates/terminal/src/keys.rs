#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    F(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
}

impl Modifiers {
    fn param(self) -> u8 {
        1 + u8::from(self.shift) + 2 * u8::from(self.alt) + 4 * u8::from(self.control)
    }
}

pub fn encode(key: Key, modifiers: Modifiers) -> Option<Vec<u8>> {
    let param = modifiers.param();
    let bytes = match key {
        Key::Up => cursor(b'A', param),
        Key::Down => cursor(b'B', param),
        Key::Right => cursor(b'C', param),
        Key::Left => cursor(b'D', param),
        Key::Home => cursor(b'H', param),
        Key::End => cursor(b'F', param),
        Key::Insert => tilde(2, param),
        Key::Delete => tilde(3, param),
        Key::PageUp => tilde(5, param),
        Key::PageDown => tilde(6, param),
        Key::F(number @ 1..=4) => function(b"PQRS"[usize::from(number - 1)], param),
        Key::F(number @ 5..=12) => tilde(
            [15, 17, 18, 19, 20, 21, 23, 24][usize::from(number - 5)],
            param,
        ),
        Key::F(_) => return None,
        Key::Tab if modifiers.shift => b"\x1b[Z".to_vec(),
        Key::Tab => alt(b"\t".to_vec(), modifiers),
        Key::Enter => alt(b"\r".to_vec(), modifiers),
        Key::Escape => alt(b"\x1b".to_vec(), modifiers),
        Key::Backspace if modifiers.control => alt(b"\x08".to_vec(), modifiers),
        Key::Backspace => alt(b"\x7f".to_vec(), modifiers),
        Key::Char(char) if modifiers.control => alt(control(char), modifiers),
        Key::Char(char) => alt(char.to_string().into_bytes(), modifiers),
    };

    Some(bytes)
}

fn cursor(letter: u8, param: u8) -> Vec<u8> {
    if param == 1 {
        return vec![0x1b, b'[', letter];
    }

    format!("\x1b[1;{param}{}", char::from(letter)).into_bytes()
}

fn function(letter: u8, param: u8) -> Vec<u8> {
    if param == 1 {
        return vec![0x1b, b'O', letter];
    }

    format!("\x1b[1;{param}{}", char::from(letter)).into_bytes()
}

fn tilde(code: u8, param: u8) -> Vec<u8> {
    if param == 1 {
        return format!("\x1b[{code}~").into_bytes();
    }

    format!("\x1b[{code};{param}~").into_bytes()
}

fn alt(mut bytes: Vec<u8>, modifiers: Modifiers) -> Vec<u8> {
    if modifiers.alt {
        bytes.insert(0, 0x1b);
    }

    bytes
}

fn control(char: char) -> Vec<u8> {
    let byte = match char {
        'a'..='z' | 'A'..='Z' => char.to_ascii_lowercase() as u8 & 0x1f,
        '@' | ' ' | '2' => 0x00,
        '[' | '3' => 0x1b,
        '\\' | '4' => 0x1c,
        ']' | '5' => 0x1d,
        '^' | '6' => 0x1e,
        '_' | '7' | '/' => 0x1f,
        '?' | '8' => 0x7f,
        _ => return char.to_string().into_bytes(),
    };

    vec![byte]
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Modifiers = Modifiers {
        shift: false,
        alt: false,
        control: false,
    };
    const SHIFT: Modifiers = Modifiers {
        shift: true,
        ..NONE
    };
    const ALT: Modifiers = Modifiers { alt: true, ..NONE };
    const CONTROL: Modifiers = Modifiers {
        control: true,
        ..NONE
    };

    fn keys(key: Key, modifiers: Modifiers) -> String {
        String::from_utf8(encode(key, modifiers).unwrap()).unwrap()
    }

    #[test]
    fn cursor_keys_use_the_short_form_until_a_modifier_is_held() {
        assert_eq!(keys(Key::Up, NONE), "\x1b[A");
        assert_eq!(keys(Key::Left, NONE), "\x1b[D");
        assert_eq!(keys(Key::Home, NONE), "\x1b[H");
        assert_eq!(keys(Key::End, NONE), "\x1b[F");
        assert_eq!(keys(Key::Right, CONTROL), "\x1b[1;5C");
        assert_eq!(
            keys(
                Key::Down,
                Modifiers {
                    shift: true,
                    alt: true,
                    ..NONE
                }
            ),
            "\x1b[1;4B"
        );
    }

    #[test]
    fn editing_and_function_keys_follow_xterm() {
        assert_eq!(keys(Key::PageUp, NONE), "\x1b[5~");
        assert_eq!(keys(Key::PageDown, SHIFT), "\x1b[6;2~");
        assert_eq!(keys(Key::Delete, NONE), "\x1b[3~");
        assert_eq!(keys(Key::F(1), NONE), "\x1bOP");
        assert_eq!(keys(Key::F(4), CONTROL), "\x1b[1;5S");
        assert_eq!(keys(Key::F(5), NONE), "\x1b[15~");
        assert_eq!(keys(Key::F(12), ALT), "\x1b[24;3~");
        assert_eq!(encode(Key::F(13), NONE), None);
    }

    #[test]
    fn control_letters_become_control_codes() {
        assert_eq!(keys(Key::Char('c'), CONTROL), "\x03");
        assert_eq!(keys(Key::Char('A'), CONTROL), "\x01");
        assert_eq!(keys(Key::Char(' '), CONTROL), "\x00");
        assert_eq!(keys(Key::Char('['), CONTROL), "\x1b");
        assert_eq!(keys(Key::Char('/'), CONTROL), "\x1f");
    }

    #[test]
    fn alt_prefixes_escape() {
        assert_eq!(keys(Key::Char('b'), ALT), "\x1bb");
        assert_eq!(
            keys(
                Key::Char('x'),
                Modifiers {
                    alt: true,
                    control: true,
                    ..NONE
                }
            ),
            "\x1b\x18"
        );
        assert_eq!(keys(Key::Backspace, ALT), "\x1b\x7f");
        assert_eq!(keys(Key::Enter, ALT), "\x1b\r");
    }

    #[test]
    fn plain_keys_send_their_usual_bytes() {
        assert_eq!(keys(Key::Char('é'), NONE), "é");
        assert_eq!(keys(Key::Backspace, NONE), "\x7f");
        assert_eq!(keys(Key::Backspace, CONTROL), "\x08");
        assert_eq!(keys(Key::Tab, NONE), "\t");
        assert_eq!(keys(Key::Tab, SHIFT), "\x1b[Z");
        assert_eq!(keys(Key::Enter, NONE), "\r");
        assert_eq!(keys(Key::Escape, NONE), "\x1b");
    }
}
