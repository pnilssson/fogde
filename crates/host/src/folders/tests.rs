use super::*;

#[tokio::test]
async fn lists_visible_folders_from_the_nearest_existing_one() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for folder in ["beta", "Alpha", ".git"] {
        fs::create_dir_all(root.join(folder)).await.unwrap();
    }
    fs::write(root.join("notes.txt"), "").await.unwrap();

    let listing = list(&root.join("missing").join("deeper")).await.unwrap();

    assert_eq!(listing.path, root.display().to_string());
    assert_eq!(
        listing
            .folders
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["Alpha", "beta"]
    );
    assert_eq!(
        listing.folders[0].path,
        root.join("Alpha").display().to_string()
    );
}

#[tokio::test]
async fn accepts_folders_to_create_but_not_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("notes.txt"), "").await.unwrap();

    assert!(usable(root).await.is_ok());
    assert!(usable(&root.join("repos")).await.is_ok());
    assert!(usable(&root.join("notes.txt")).await.is_err());
}

#[tokio::test]
async fn creates_one_named_folder_inside_its_parent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let created = create(root, "app").await.unwrap();
    let again = create(root, "app").await.unwrap();

    assert_eq!(created, root.join("app"));
    assert_eq!(again, created);
    assert!(created.is_dir());
    for name in ["", ".", "..", "../app", "app/src", "/app"] {
        assert!(create(root, name).await.is_err(), "{name}");
    }
    assert!(!root.parent().unwrap().join("app").exists());
}

#[tokio::test]
async fn removes_only_empty_folders() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let empty = create(root, "empty").await.unwrap();
    let full = create(root, "full").await.unwrap();
    fs::write(full.join(".keep"), "").await.unwrap();

    remove(&empty).await.unwrap();
    let refused = remove(&full).await.unwrap_err();

    assert!(!empty.exists());
    assert_eq!(refused.kind(), io::ErrorKind::DirectoryNotEmpty);
    assert!(full.join(".keep").exists());
    assert!(remove(&root.join("notes.txt")).await.is_err());
}

#[test]
fn compares_folders_by_their_parts() {
    let repo = std::env::temp_dir().join("app");

    assert!(inside(&repo.join("src"), &repo));
    assert!(inside(&repo, &repo));
    assert!(!inside(&repo, &repo.join("src")));
    assert!(!inside(&std::env::temp_dir().join("app-old"), &repo));
}

#[cfg(windows)]
#[test]
fn ignores_case_and_separators_on_windows() {
    assert!(inside(
        Path::new(r"c:\users\dev\App\src\"),
        Path::new("C:/Users/dev/app")
    ));
}

#[test]
fn requires_a_full_path() {
    assert!(absolute("repos").is_err());
    assert!(absolute(&std::env::temp_dir().display().to_string()).is_ok());
}

#[tokio::test]
async fn sees_the_same_folder_through_a_detour() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("app");
    fs::create_dir_all(repo.join("src")).await.unwrap();

    assert!(identical(&repo.join("src").join(".."), &repo).await);
    assert!(!identical(&repo.join("src"), &repo).await);
    assert!(identical(&repo.join("missing"), &repo.join("missing")).await);
}
