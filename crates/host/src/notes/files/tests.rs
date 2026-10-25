use std::io;

use super::super::list;
use super::super::tests::create;

#[tokio::test]
async fn creates_notes_and_folders_inside_the_notes_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("notes");
    std::fs::create_dir_all(root.join("app")).unwrap();

    let note = super::note(&root, &root.join("app"), " Login flow ")
        .await
        .unwrap();
    let named = super::note(&root, &root, "plan.md").await.unwrap();
    let folder = super::folder(&root, &root, "specs").await.unwrap();

    assert_eq!(note, root.join("app").join("Login flow.md"));
    assert_eq!(std::fs::read_to_string(&note).unwrap(), "");
    assert_eq!(named, root.join("plan.md"));
    assert!(folder.is_dir());

    let again = super::note(&root, &root.join("app"), "Login flow")
        .await
        .unwrap_err();
    assert_eq!(again.kind(), io::ErrorKind::AlreadyExists);
    assert!(super::folder(&root, &root, "specs").await.is_err());
    for name in ["", ".hidden", "../outside", "app/plan"] {
        assert!(super::note(&root, &root, name).await.is_err(), "{name}");
    }
    assert!(super::note(&root, dir.path(), "outside").await.is_err());
    assert!(!dir.path().join("outside.md").exists());
}

#[tokio::test]
async fn renames_a_note_in_its_folder_without_replacing_another() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let plan = root.join("app").join("plan.md");
    create(&plan);
    create(&root.join("app").join("backlog.md"));

    let renamed = super::rename(&root, &plan, "Login plan").await.unwrap();
    let cased = super::rename(&root, &renamed, "login plan.md")
        .await
        .unwrap();
    let taken = super::rename(&root, &cased, "backlog").await;

    assert_eq!(renamed, root.join("app").join("Login plan.md"));
    assert_eq!(cased, root.join("app").join("login plan.md"));
    assert!(cased.exists());
    assert!(!plan.exists());
    assert_eq!(taken.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        std::fs::read_to_string(root.join("app").join("backlog.md")).unwrap(),
        "# Note"
    );
    assert!(super::rename(&root, &cased, "../plan").await.is_err());
}

#[tokio::test]
async fn deletes_a_note_into_the_trash_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    for folder in ["app", "web"] {
        create(&root.join(folder).join("plan.md"));
    }

    super::trash(&root, &root.join("app").join("plan.md"))
        .await
        .unwrap();
    super::trash(&root, &root.join("web").join("plan.md"))
        .await
        .unwrap();

    assert!(!root.join("app").join("plan.md").exists());
    assert!(root.join(".trash").join("plan.md").exists());
    assert!(root.join(".trash").join("plan 2.md").exists());
    let listed = list(&root).unwrap();
    assert_eq!(listed.notes, Vec::new());
    assert!(super::trash(&root, &root.join("missing.md")).await.is_err());
}

#[tokio::test]
async fn renames_a_folder_with_its_notes_without_replacing_another() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create(&root.join("app").join("plan.md"));
    create(&root.join("web").join("plan.md"));

    let renamed = super::rename(&root, &root.join("app"), "specs")
        .await
        .unwrap();
    let taken = super::rename(&root, &renamed, "web").await;

    assert_eq!(renamed, root.join("specs"));
    assert!(renamed.join("plan.md").exists());
    assert!(!root.join("app").exists());
    assert_eq!(taken.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
    assert!(root.join("web").join("plan.md").exists());
    for name in ["../specs", ".hidden"] {
        assert!(
            super::rename(&root, &renamed, name).await.is_err(),
            "{name}"
        );
    }
    assert!(super::rename(&root, &root, "other").await.is_err());
    let up = renamed.join("sub").join("..");
    std::fs::create_dir_all(renamed.join("sub")).unwrap();
    assert!(super::rename(&root, &up, "docs").await.is_err());
    assert!(renamed.join("sub").exists());
}

#[tokio::test]
async fn deletes_a_folder_into_the_trash_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    create(&root.join("app").join("plan.md"));
    create(&root.join("web").join("app").join("plan.md"));

    super::trash(&root, &root.join("app")).await.unwrap();
    super::trash(&root, &root.join("web").join("app"))
        .await
        .unwrap();

    assert!(!root.join("app").exists());
    assert!(root.join(".trash").join("app").join("plan.md").exists());
    assert!(root.join(".trash").join("app 2").join("plan.md").exists());
    assert_eq!(list(&root).unwrap().notes, Vec::new());
    std::fs::create_dir_all(root.join("web").join("sub")).unwrap();
    for path in [
        root.clone(),
        root.join(".trash"),
        root.join(".trash").join("app"),
        root.join("web").join("sub").join(".."),
    ] {
        assert!(
            super::trash(&root, &path).await.is_err(),
            "{}",
            path.display()
        );
    }
    assert!(root.join("web").join("sub").exists());
}
