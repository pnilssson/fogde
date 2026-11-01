use std::collections::HashSet;
use std::time::Duration;

use tokio::sync::broadcast;

use super::watch::{classify, known, start, visible};
use super::*;

pub(super) fn create(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "# Note").unwrap();
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

#[test]
fn lists_markdown_notes_in_visible_folders() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for file in [
        "login.md",
        "Backlog.MD",
        "readme.txt",
        ".obsidian/workspace.md",
        ".trash/old.md",
        "app/plan.md",
        "app/.git/notes.md",
    ] {
        create(&root.join(file));
    }
    std::fs::create_dir_all(root.join("empty")).unwrap();

    let listing = list(root).unwrap();

    assert_eq!(listing.root, shown(root));
    assert_eq!(
        listing.folders,
        [shown(&root.join("app")), shown(&root.join("empty"))]
    );
    assert_eq!(
        listing
            .notes
            .iter()
            .map(|note| (note.name.as_str(), note.folder.clone()))
            .collect::<Vec<_>>(),
        [
            ("plan", shown(&root.join("app"))),
            ("Backlog", shown(root)),
            ("login", shown(root)),
        ]
    );
    let plan = &listing.notes[0];
    assert_eq!(plan.path, shown(&root.join("app").join("plan.md")));
    assert!(plan.modified > 0);
    assert!(plan.created > 0);
}

#[tokio::test]
async fn reads_only_notes_inside_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("notes");
    create(&root.join("app").join("plan.md"));
    create(&root.join("readme.txt"));
    create(&root.join(".trash").join("old.md"));
    create(&dir.path().join("outside.md"));

    let plan = read(&root, &root.join("app").join("plan.md"))
        .await
        .unwrap();

    assert_eq!(plan.text, "# Note");
    for path in [
        root.join("readme.txt"),
        root.join(".trash").join("old.md"),
        root.join("app"),
        dir.path().join("outside.md"),
        root.join("app").join("..").join("..").join("outside.md"),
        root.join("missing.md"),
    ] {
        assert!(read(&root, &path).await.is_err(), "{}", path.display());
    }
}

#[tokio::test]
async fn writes_a_note_only_over_the_version_it_was_read_at() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("notes");
    let path = root.join("plan.md");
    create(&path);
    create(&dir.path().join("outside.md"));
    std::fs::write(&path, "# Plan\r\n\r\n- one\r\n").unwrap();
    let loaded = read(&root, &path).await.unwrap();

    let edit = "# Plan\r\n\r\n- two\r\n";
    let saved = write(&root, &path, edit, &loaded.version).await.unwrap();
    let stale = write(&root, &path, "# Stale", &loaded.version)
        .await
        .unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), edit);
    let Written::Saved { version } = saved else {
        panic!("the note was not saved");
    };
    assert_eq!(read(&root, &path).await.unwrap().version, version);
    assert_eq!(stale, Written::Changed);
    let outside = dir.path().join("outside.md");
    assert!(write(&root, &outside, "", &version).await.is_err());
}

#[tokio::test]
async fn saves_a_note_in_one_step_without_leaving_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("plan.md");
    std::fs::write(&path, "# Plan\n\n- login\n").unwrap();
    let earlier = root.join(".earlier");
    std::fs::hard_link(&path, &earlier).unwrap();
    let loaded = read(&root, &path).await.unwrap();

    write(&root, &path, "# Plan\n\n- signup\n", &loaded.version)
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# Plan\n\n- signup\n"
    );
    assert_eq!(
        std::fs::read_to_string(&earlier).unwrap(),
        "# Plan\n\n- login\n"
    );
    let mut names: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, [".earlier", "plan.md"]);
}

#[cfg(unix)]
#[tokio::test]
async fn keeps_the_mode_of_a_saved_note() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("plan.md");
    std::fs::write(
        &path, "# Plan
",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let loaded = read(&root, &path).await.unwrap();

    write(
        &root,
        &path,
        "# Plan

- login
",
        &loaded.version,
    )
    .await
    .unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn leaves_a_read_only_note_alone() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("plan.md");
    std::fs::write(&path, "# Plan\n").unwrap();
    let writable = std::fs::metadata(&path).unwrap().permissions();
    let mut locked = writable.clone();
    locked.set_readonly(true);
    std::fs::set_permissions(&path, locked).unwrap();
    let loaded = read(&root, &path).await.unwrap();

    let written = write(&root, &path, "# Plan\n\n- login\n", &loaded.version).await;

    assert!(written.is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# Plan\n");
    std::fs::set_permissions(&path, writable).unwrap();
}

#[tokio::test]
async fn refuses_to_read_a_note_that_is_not_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plan.md");
    std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();

    assert!(read(dir.path(), &path).await.is_err());
}

#[test]
fn ignores_changes_in_dot_folders() {
    let root = std::env::temp_dir().join("notes");
    let roots = [root.clone()];

    assert!(visible(&root.join("app").join("plan.md"), &roots));
    assert!(visible(&root, &roots));
    assert!(!visible(
        &root.join(".obsidian").join("workspace.json"),
        &roots
    ));
    assert!(!visible(
        &root.join("app").join(".git").join("index"),
        &roots
    ));
}

#[tokio::test]
async fn reports_an_edited_note() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create(&root.join("plan.md"));
    let (sender, mut changes) = broadcast::channel(16);

    let _watching = start(root.clone(), sender).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    std::fs::write(root.join("plan.md"), "# Edited").unwrap();

    let update = tokio::time::timeout(Duration::from_secs(10), changes.recv())
        .await
        .unwrap()
        .unwrap();

    assert!(!update.listing);
    assert_eq!(update.notes.len(), 1);
    assert_eq!(update.notes[0].path, shown(&root.join("plan.md")));
}

#[test]
fn tells_an_edit_from_a_note_that_came_or_went() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let plan = root.join("app").join("plan.md");
    create(&plan);
    let mut known = known(&root);
    let changed = |paths: &[&Path], known: &mut HashSet<PathBuf>| {
        classify(paths.iter().map(|path| path.to_path_buf()).collect(), known)
    };

    std::fs::write(&plan, "# Edited").unwrap();
    let edited = changed(&[&plan, &root.join("app"), &root], &mut known);
    assert!(!edited.listing);
    assert_eq!(edited.notes.len(), 1);

    let idea = root.join("app").join("idea.md");
    create(&idea);
    assert!(changed(&[&idea], &mut known).listing);
    assert!(!changed(&[&idea], &mut known).listing);

    std::fs::remove_dir_all(root.join("app")).unwrap();
    assert!(changed(&[&root.join("app")], &mut known).listing);
    create(&plan);
    assert!(changed(&[&plan], &mut known).listing);

    std::fs::write(root.join("image.png"), "png").unwrap();
    let other = changed(&[&root.join("image.png")], &mut known);
    assert!(!other.listing);
    assert_eq!(other.notes, []);
}
