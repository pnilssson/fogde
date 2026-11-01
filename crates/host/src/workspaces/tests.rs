use super::*;

fn checkout(path: &Path, main: bool) -> Checkout {
    Checkout {
        path: path.display().to_string(),
        branch: None,
        main,
    }
}

#[test]
fn finds_the_linked_worktree_of_a_folder() {
    let root = std::env::temp_dir();
    let repo = root.join("app");
    let nested = repo.join("worktrees").join("login");
    let checkouts = [checkout(&repo, true), checkout(&nested, false)];

    assert_eq!(worktree_of(&repo.join("crates"), &checkouts), None);
    assert_eq!(
        worktree_of(&nested.join("src"), &checkouts),
        Some(nested.display().to_string())
    );
    assert_eq!(worktree_of(&root.join("app-old"), &checkouts), None);
}

#[test]
fn a_stored_worktree_takes_the_spelling_git_lists() {
    let repo = std::env::temp_dir().join("app");
    let login = repo.join("worktrees").join("login");
    let checkouts = [checkout(&repo, true), checkout(&login, false)];
    let stored = format!("{}{}", login.display(), std::path::MAIN_SEPARATOR);

    assert_eq!(
        listed(&stored, &checkouts),
        Some(login.display().to_string())
    );
    assert_eq!(listed(&repo.display().to_string(), &checkouts), None);
}
