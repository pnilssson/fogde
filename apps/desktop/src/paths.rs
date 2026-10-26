fn windows(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn parts(path: &str) -> Vec<String> {
    let lower = windows(path);
    path.split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            if lower {
                part.to_lowercase()
            } else {
                part.to_owned()
            }
        })
        .collect()
}

pub fn inside(path: &str, root: &str) -> bool {
    let outer = parts(root);
    let inner = parts(path);

    !outer.is_empty() && inner.starts_with(&outer)
}

pub fn name(path: &str) -> &str {
    path.split(['/', '\\'])
        .rfind(|part| !part.is_empty())
        .unwrap_or(path)
}

pub fn relative(path: &str, root: &str) -> Option<String> {
    if !inside(path, root) {
        return None;
    }

    let separator = if path.contains('\\') { "\\" } else { "/" };
    let below: Vec<&str> = path
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .skip(parts(root).len())
        .collect();

    Some(below.join(separator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_folder_inside_another_by_its_parts() {
        assert!(inside("/worktrees/app/login/src", "/worktrees/app/login"));
        assert!(inside("/worktrees/app/login", "/worktrees/app/login/"));
        assert!(!inside("/worktrees/app/login-old", "/worktrees/app/login"));
        assert!(!inside("/worktrees/app", "/worktrees/app/login"));
        assert!(!inside("", "/worktrees/app/login"));
    }

    #[test]
    fn ignores_case_and_separators_in_windows_paths() {
        assert!(inside("c:\\users\\dev\\Login\\src", "C:/Users/dev/login"));
    }

    #[test]
    fn gives_the_path_below_a_folder_in_the_paths_own_spelling() {
        assert_eq!(
            relative("/repos/app/src/login", "/repos/app").as_deref(),
            Some("src/login")
        );
        assert_eq!(relative("/repos/app/", "/repos/app").as_deref(), Some(""));
        assert_eq!(
            relative("C:\\Repos\\App\\Src", "c:/repos/app").as_deref(),
            Some("Src")
        );
        assert_eq!(relative("/repos/api", "/repos/app"), None);
    }

    #[test]
    fn names_a_folder_by_its_last_part() {
        assert_eq!(name("/repos/app/"), "app");
        assert_eq!(name("C:\\repos\\api"), "api");
    }
}
