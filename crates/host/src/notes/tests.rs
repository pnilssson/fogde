use std::time::Duration;

use tokio::sync::broadcast;

use super::watch::{start, visible};
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

    tokio::time::timeout(Duration::from_secs(10), changes.recv())
        .await
        .unwrap()
        .unwrap();
}
