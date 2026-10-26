use protocol::notes::Note;

fn parts(path: &str) -> Vec<String> {
    let bytes = path.as_bytes();
    let windows = (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || path.contains('\\');
    let mut resolved: Vec<String> = Vec::new();
    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                resolved.pop();
            }
            part => resolved.push(if windows {
                part.to_lowercase()
            } else {
                part.to_owned()
            }),
        }
    }
    resolved
}

fn key(path: &str) -> String {
    parts(path).join("/")
}

fn without_extension(path: &str) -> &str {
    if path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md") {
        &path[..path.len() - 3]
    } else {
        path
    }
}

fn closest<'a>(mut found: Vec<&'a Note>, from: Option<&Note>) -> Option<&'a Note> {
    let here = from.map(|note| key(&note.folder));
    found.sort_by(|a, b| {
        let near = |note: &Note| Some(key(&note.folder)) == here;
        near(b)
            .cmp(&near(a))
            .then(parts(&a.path).len().cmp(&parts(&b.path).len()))
            .then(a.path.cmp(&b.path))
    });
    found.into_iter().next()
}

pub fn resolve_wikilink<'a>(
    target: &str,
    notes: &'a [Note],
    from: Option<&Note>,
) -> Option<&'a Note> {
    let target = target.split('#').next().unwrap_or_default();
    let wanted = key(without_extension(target)).to_lowercase();
    if wanted.is_empty() {
        return None;
    }

    let found = notes
        .iter()
        .filter(|note| {
            let path = key(without_extension(&note.path)).to_lowercase();
            path == wanted || path.ends_with(&format!("/{wanted}"))
        })
        .collect();
    closest(found, from)
}

fn decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = text.get(index + 1..index + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok()
}

pub fn resolve_href<'a>(href: &str, notes: &'a [Note], from: &Note) -> Option<&'a Note> {
    let path = decode(href.split('#').next().unwrap_or_default())?;
    if !(path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md")) {
        return None;
    }

    let wanted = key(&format!("{}/{path}", from.folder));
    notes
        .iter()
        .find(|note| key(&note.path) == wanted)
        .or_else(|| resolve_wikilink(&path, notes, Some(from)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(path: &str) -> Note {
        let at = path.rfind('/').unwrap();
        Note {
            name: path[at + 1..].trim_end_matches(".md").into(),
            path: path.into(),
            folder: path[..at].into(),
            modified: 0,
            created: 0,
        }
    }

    fn notes() -> Vec<Note> {
        [
            "/notes/backlog.md",
            "/notes/web/plan.md",
            "/notes/app/plan.md",
            "/notes/app/specs/Login flow.md",
        ]
        .map(note)
        .to_vec()
    }

    fn path(found: Option<&Note>) -> Option<&str> {
        found.map(|note| note.path.as_str())
    }

    #[test]
    fn resolves_a_wikilink_by_name_preferring_the_linking_notes_folder() {
        let notes = notes();
        let (backlog, plan, login) = (&notes[0], &notes[2], &notes[3]);
        assert_eq!(
            path(resolve_wikilink("login FLOW", &notes, Some(backlog))),
            Some("/notes/app/specs/Login flow.md")
        );
        assert_eq!(
            path(resolve_wikilink(
                "plan",
                &notes,
                Some(&note("/notes/web/ideas.md"))
            )),
            Some("/notes/web/plan.md")
        );
        assert_eq!(
            path(resolve_wikilink("plan#Next", &notes, Some(login))),
            Some("/notes/app/plan.md")
        );
        assert_eq!(
            path(resolve_wikilink("web/plan.md", &notes, Some(plan))),
            Some("/notes/web/plan.md")
        );
        assert_eq!(resolve_wikilink("missing", &notes, Some(plan)), None);
    }

    #[test]
    fn prefers_the_shortest_path_when_no_folder_matches() {
        let both = [
            note("/notes/archive/old/backlog.md"),
            note("/notes/backlog.md"),
        ];
        assert_eq!(
            path(resolve_wikilink(
                "backlog",
                &both,
                Some(&note("/notes/app/plan.md"))
            )),
            Some("/notes/backlog.md")
        );
    }

    #[test]
    fn resolves_relative_markdown_links_from_the_notes_folder() {
        let notes = notes();
        let (backlog, plan) = (&notes[0], &notes[2]);
        assert_eq!(
            path(resolve_href("specs/Login%20flow.md", &notes, plan)),
            Some("/notes/app/specs/Login flow.md")
        );
        assert_eq!(
            path(resolve_href("../web/plan.md#Next", &notes, plan)),
            Some("/notes/web/plan.md")
        );
        assert_eq!(
            path(resolve_href("Login%20flow.md", &notes, backlog)),
            Some("/notes/app/specs/Login flow.md")
        );
        assert_eq!(resolve_href("https://example.com", &notes, plan), None);
        assert_eq!(resolve_href("%E0%A4%A.md", &notes, plan), None);
    }

    #[test]
    fn matches_windows_paths_whatever_their_case_and_separators() {
        let windows = Note {
            name: "plan".into(),
            path: r"C:\Users\dev\notes\app\plan.md".into(),
            folder: r"C:\Users\dev\notes\app".into(),
            modified: 0,
            created: 0,
        };
        let from = Note {
            name: "ideas".into(),
            path: r"C:\Users\dev\notes\app\ideas.md".into(),
            ..windows.clone()
        };
        let notes = [windows];

        assert!(resolve_wikilink("APP/Plan", &notes, Some(&from)).is_some());
        assert!(resolve_href("./Plan.md", &notes, &from).is_some());
    }
}
