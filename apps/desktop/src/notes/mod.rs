pub mod editor;
pub mod links;
pub mod live;
pub mod merge;
mod name;
pub mod palette;
mod pane;
pub mod recent;
pub mod renames;
pub mod session;
pub mod tree;
pub mod view;
pub mod wikilinks;

use std::fmt::Write as _;

pub fn key() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        bytes = nanos.to_le_bytes();
    }
    bytes.iter().fold(String::from("notes:"), |mut key, byte| {
        let _ = write!(key, "{byte:02x}");
        key
    })
}
