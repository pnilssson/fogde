use protocol::notes::{Note, NoteList};

use super::renames::moved;
use crate::paths;

pub const KEPT: usize = 20;
pub const SHOWN: usize = 8;
pub const FOUND: usize = 20;

pub fn viewed(list: &[String], path: &str) -> Vec<String> {
    std::iter::once(path.to_owned())
        .chain(list.iter().filter(|item| *item != path).cloned())
        .take(KEPT)
        .collect()
}

pub fn follow(list: &[String], from: &str, to: &str) -> Vec<String> {
    list.iter()
        .map(|path| moved(path, from, to).unwrap_or_else(|| path.clone()))
        .collect()
}

pub fn recent<'a>(notes: &'a [Note], viewed: &[String]) -> Vec<&'a Note> {
    let mut seen: Vec<&Note> = viewed
        .iter()
        .filter_map(|path| notes.iter().find(|note| note.path == *path))
        .collect();
    if seen.len() >= SHOWN {
        seen.truncate(SHOWN);
        return seen;
    }

    let mut rest: Vec<&Note> = notes
        .iter()
        .filter(|note| !seen.iter().any(|known| known.path == note.path))
        .collect();
    rest.sort_by_key(|note| std::cmp::Reverse(note.modified));
    seen.extend(rest);
    seen.truncate(SHOWN);
    seen
}

pub fn matching<'a>(list: &'a NoteList, search: &str) -> Vec<&'a Note> {
    let query = search.trim().to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut found: Vec<(usize, &Note)> = list
        .notes
        .iter()
        .filter_map(|note| {
            let name = note.name.to_lowercase();
            let folder = paths::relative(&note.folder, &list.root)
                .unwrap_or_default()
                .to_lowercase();
            let haystack = format!("{name} {folder}");
            words.iter().all(|word| haystack.contains(word)).then(|| {
                let rank = if name.starts_with(&query) {
                    0
                } else if name.contains(&query) {
                    1
                } else {
                    2
                };
                (rank, note)
            })
        })
        .collect();
    found.sort_by(|(rank_a, a), (rank_b, b)| rank_a.cmp(rank_b).then(b.modified.cmp(&a.modified)));
    found
        .into_iter()
        .take(FOUND)
        .map(|(_, note)| note)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(folder: &str, name: &str, modified: i64) -> Note {
        Note {
            name: name.into(),
            path: format!("{folder}/{name}.md"),
            folder: folder.into(),
            modified,
            created: modified,
        }
    }

    #[test]
    fn viewing_a_note_moves_it_to_the_front_and_keeps_twenty() {
        let many: Vec<String> = (0..25).map(|index| format!("/notes/{index}.md")).collect();
        let list = viewed(&many, "/notes/3.md");
        assert_eq!(list[0], "/notes/3.md");
        assert_eq!(list.len(), KEPT);
        assert_eq!(list.iter().filter(|path| *path == "/notes/3.md").count(), 1);
    }

    #[test]
    fn recent_notes_fill_up_with_the_last_modified() {
        let notes = [
            note("/notes", "a", 1),
            note("/notes", "b", 3),
            note("/notes", "c", 2),
        ];
        let shown: Vec<&str> = recent(&notes, &["/notes/a.md".into()])
            .iter()
            .map(|note| note.name.as_str())
            .collect();
        assert_eq!(shown, ["a", "b", "c"]);
    }

    #[test]
    fn search_ranks_prefix_then_contains_then_recent() {
        let list = NoteList {
            root: "/notes".into(),
            folders: vec!["/notes/app".into()],
            notes: vec![
                note("/notes/app", "readme", 9),
                note("/notes", "login plan", 1),
                note("/notes", "plan", 2),
                note("/notes", "my plan", 5),
            ],
        };
        let names: Vec<&str> = matching(&list, "plan")
            .iter()
            .map(|note| note.name.as_str())
            .collect();
        assert_eq!(names, ["plan", "my plan", "login plan"]);
        let names: Vec<&str> = matching(&list, "app read")
            .iter()
            .map(|note| note.name.as_str())
            .collect();
        assert_eq!(names, ["readme"]);
    }

    #[test]
    fn renames_follow_into_recent_notes() {
        let list = follow(&["/notes/app/plan.md".into()], "/notes/app", "/notes/web");
        assert_eq!(list, ["/notes/web/plan.md"]);
    }
}
