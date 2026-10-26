use protocol::terminals::{Key, Scroll};

pub fn key(key: Key) -> herdr_client::Key {
    match key {
        Key::ShiftEnter => herdr_client::Key::ShiftEnter,
        Key::CtrlEnter => herdr_client::Key::CtrlEnter,
    }
}

pub fn scroll(scroll: Scroll) -> herdr_client::Scroll {
    match scroll {
        Scroll::Up => herdr_client::Scroll::Up,
        Scroll::Down => herdr_client::Scroll::Down,
    }
}
