use std::cmp::Ordering;
use std::collections::HashMap;

use protocol::notes::{Note, NoteList};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Order {
    #[default]
    NameAsc,
    NameDesc,
    ModifiedDesc,
    ModifiedAsc,
    CreatedDesc,
    CreatedAsc,
}

impl Order {
    pub const ALL: [Order; 6] = [
        Order::NameAsc,
        Order::NameDesc,
        Order::ModifiedDesc,
        Order::ModifiedAsc,
        Order::CreatedDesc,
        Order::CreatedAsc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Order::NameAsc => "Name (A to Z)",
            Order::NameDesc => "Name (Z to A)",
            Order::ModifiedDesc => "Modified (newest first)",
            Order::ModifiedAsc => "Modified (oldest first)",
            Order::CreatedDesc => "Created (newest first)",
            Order::CreatedAsc => "Created (oldest first)",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    pub name: String,
    pub path: String,
    pub folders: Vec<Folder>,
    pub notes: Vec<Note>,
}

pub fn compare_names(a: &str, b: &str) -> Ordering {
    fn chunks(text: &str) -> Vec<(bool, String)> {
        let mut out: Vec<(bool, String)> = Vec::new();
        for char in text.chars() {
            let digit = char.is_ascii_digit();
            match out.last_mut() {
                Some((kind, chunk)) if *kind == digit => chunk.push(char),
                _ => out.push((digit, char.to_string())),
            }
        }
        out
    }

    let (left, right) = (chunks(a), chunks(b));
    for ((digit_a, a), (digit_b, b)) in left.iter().zip(&right) {
        let order = if *digit_a && *digit_b {
            let (a, b) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
            a.len().cmp(&b.len()).then_with(|| a.cmp(b))
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    left.len().cmp(&right.len())
}

fn trimmed(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        &path[..path.len().min(1)]
    } else {
        trimmed
    }
}

fn parent(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(at) if at > 0 => &path[..at],
        _ => path,
    }
}

fn compare_notes(order: Order, a: &Note, b: &Note) -> Ordering {
    let by_name = compare_names(&a.name, &b.name);
    match order {
        Order::NameAsc => by_name,
        Order::NameDesc => by_name.reverse(),
        Order::ModifiedDesc => b.modified.cmp(&a.modified).then(by_name),
        Order::ModifiedAsc => a.modified.cmp(&b.modified).then(by_name),
        Order::CreatedDesc => b.created.cmp(&a.created).then(by_name),
        Order::CreatedAsc => a.created.cmp(&b.created).then(by_name),
    }
}

fn sort(folder: &mut Folder, order: Order) {
    folder.folders.sort_by(|a, b| {
        let order_by_name = compare_names(&a.name, &b.name);
        if order == Order::NameDesc {
            order_by_name.reverse()
        } else {
            order_by_name
        }
    });
    folder.notes.sort_by(|a, b| compare_notes(order, a, b));
    for inner in &mut folder.folders {
        sort(inner, order);
    }
}

fn attach(
    key: &str,
    folders: &mut HashMap<String, Folder>,
    children: &HashMap<String, Vec<String>>,
) -> Vec<Folder> {
    children
        .get(key)
        .into_iter()
        .flatten()
        .filter_map(|child| {
            let inner = attach(child, folders, children);
            folders.remove(child).map(|mut folder| {
                folder.folders = inner;
                folder
            })
        })
        .collect()
}

pub fn build(list: &NoteList, order: Order) -> Folder {
    let root_key = trimmed(&list.root).to_owned();
    let mut folders: HashMap<String, Folder> = HashMap::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for path in &list.folders {
        let key = trimmed(path).to_owned();
        folders.insert(
            key.clone(),
            Folder {
                name: paths::name(path).to_owned(),
                path: path.clone(),
                folders: Vec::new(),
                notes: Vec::new(),
            },
        );
        let above = trimmed(parent(&key)).to_owned();
        if above != key {
            children.entry(above).or_default().push(key);
        }
    }
    let mut root = Folder {
        name: paths::name(&list.root).to_owned(),
        path: list.root.clone(),
        folders: Vec::new(),
        notes: Vec::new(),
    };
    for note in &list.notes {
        let key = trimmed(&note.folder);
        if key == root_key {
            root.notes.push(note.clone());
        } else if let Some(folder) = folders.get_mut(key) {
            folder.notes.push(note.clone());
        }
    }

    root.folders = attach(&root_key, &mut folders, &children);
    sort(&mut root, order);
    root
}

pub fn folders_above(path: &str, root: &str) -> Vec<String> {
    let top = trimmed(root);
    let mut above = Vec::new();
    let mut current = trimmed(parent(path));
    while current.len() > top.len() {
        above.push(current.to_owned());
        let next = trimmed(parent(current));
        if next == current {
            break;
        }
        current = next;
    }
    above
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(folder: &str, name: &str, modified: i64) -> Note {
        Note {
            name: name.into(),
            path: format!("{folder}\\{name}.md"),
            folder: folder.into(),
            modified,
            created: modified,
        }
    }

    fn list() -> NoteList {
        let root = r"C:\notes";
        NoteList {
            root: root.into(),
            folders: vec![
                format!(r"{root}\app"),
                format!(r"{root}\app\specs"),
                format!(r"{root}\Web"),
            ],
            notes: vec![
                note(root, "note 10", 1),
                note(root, "Note 2", 3),
                note(root, "backlog", 2),
                note(&format!(r"{root}\app"), "plan", 1),
                note(&format!(r"{root}\app\specs"), "login", 1),
            ],
        }
    }

    fn names(notes: &[Note]) -> Vec<&str> {
        notes.iter().map(|note| note.name.as_str()).collect()
    }

    #[test]
    fn builds_the_folder_tree_sorted_by_name_like_obsidian() {
        let tree = build(&list(), Order::NameAsc);

        assert_eq!(tree.name, "notes");
        assert_eq!(names(&tree.notes), ["backlog", "Note 2", "note 10"]);
        assert_eq!(tree.folders.len(), 2);
        assert_eq!(tree.folders[0].name, "app");
        assert_eq!(names(&tree.folders[0].notes), ["plan"]);
        assert_eq!(tree.folders[0].folders[0].name, "specs");
        assert_eq!(names(&tree.folders[0].folders[0].notes), ["login"]);
        assert_eq!(tree.folders[1].name, "Web");
    }

    #[test]
    fn sorts_notes_by_time_while_folders_stay_by_name() {
        let tree = build(&list(), Order::ModifiedDesc);

        assert_eq!(names(&tree.notes), ["Note 2", "backlog", "note 10"]);
        assert_eq!(
            tree.folders
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            ["app", "Web"]
        );
        let reversed = build(&list(), Order::NameDesc);
        assert_eq!(
            reversed
                .folders
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            ["Web", "app"]
        );
    }

    #[test]
    fn lists_the_folders_above_a_note_up_to_the_root() {
        assert_eq!(
            folders_above(r"C:\notes\app\specs\login.md", r"C:\notes"),
            [r"C:\notes\app\specs", r"C:\notes\app"]
        );
        assert_eq!(
            folders_above("/notes/plan.md", "/notes/"),
            Vec::<String>::new()
        );
    }
}
