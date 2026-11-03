use std::fs::File;

use super::*;

#[tokio::test]
async fn saves_the_file_under_its_name_and_removes_week_old_ones() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");
    std::fs::create_dir(&files).unwrap();
    let old = files.join("old.png");
    let recent = files.join("recent.png");
    File::create(&old)
        .unwrap()
        .set_modified(SystemTime::now() - KEPT - Duration::from_secs(60))
        .unwrap();
    File::create(&recent).unwrap();

    let saved = save(&files, "photo.jpg", STANDARD.encode(b"jpeg"))
        .await
        .unwrap();

    let name = saved.file_name().unwrap().to_string_lossy();
    assert!(name.ends_with("-photo.jpg"), "{name}");
    assert_eq!(std::fs::read(&saved).unwrap(), b"jpeg");
    for _ in 0..100 {
        if !old.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!old.exists());
    assert!(recent.exists());
}

#[tokio::test]
async fn a_repaste_of_a_week_old_file_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");
    let saved = save(&files, "image.png", STANDARD.encode(b"png"))
        .await
        .unwrap();
    File::options()
        .write(true)
        .open(&saved)
        .unwrap()
        .set_modified(SystemTime::now() - KEPT - Duration::from_secs(60))
        .unwrap();

    let again = save(&files, "image.png", STANDARD.encode(b"png"))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(again, saved);
    assert!(saved.exists());
}

#[tokio::test]
async fn pastes_of_the_same_file_share_one_copy() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");

    let first = save(&files, "notes.md", STANDARD.encode(b"md"))
        .await
        .unwrap();
    let second = save(&files, "notes.md", STANDARD.encode(b"md"))
        .await
        .unwrap();
    let other = save(&files, "notes.md", STANDARD.encode(b"other"))
        .await
        .unwrap();

    assert_eq!(first, second);
    assert_ne!(first, other);
    assert_eq!(std::fs::read_dir(&files).unwrap().count(), 2);
}

#[tokio::test]
async fn a_repaste_keeps_the_file_and_refreshes_its_age() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");
    let day = SystemTime::now() - Duration::from_hours(24);
    let saved = save(&files, "image.png", STANDARD.encode(b"png"))
        .await
        .unwrap();
    std::fs::write(&saved, b"kept").unwrap();
    File::options()
        .write(true)
        .open(&saved)
        .unwrap()
        .set_modified(day)
        .unwrap();

    let again = save(&files, "image.png", STANDARD.encode(b"png"))
        .await
        .unwrap();
    let modified = std::fs::metadata(&saved).unwrap().modified().unwrap();

    assert_eq!(again, saved);
    assert_eq!(std::fs::read(&saved).unwrap(), b"kept");
    assert!(modified > day + Duration::from_hours(23));
    assert_eq!(std::fs::read_dir(&files).unwrap().count(), 1);
}

#[tokio::test]
async fn refuses_files_over_the_limit_and_bad_data() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");
    let large = "A".repeat(FILE_LIMIT.div_ceil(3) * 4 + 4);

    assert_eq!(
        save(&files, "large.bin", large).await.unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        save(&files, "bad.txt", "not base64!".to_owned())
            .await
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert!(!files.exists());
}

#[tokio::test]
async fn keeps_a_dropped_name_inside_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let files = dir.path().join("files");

    for name in ["../../etc/passwd", r"..\..\Windows\win.ini", "..", ""] {
        let saved = save(&files, name, STANDARD.encode(name)).await.unwrap();

        assert_eq!(saved.parent(), Some(files.as_path()), "{name}");
    }
}

#[test]
fn names_a_copy_after_the_dropped_file() {
    assert_eq!(clean("feature login.md"), "feature-login.md");
    assert_eq!(clean("/repos/app/src/main.rs"), "main.rs");
    assert_eq!(clean(r"C:\repos\app\Cargo.toml"), "Cargo.toml");
    assert_eq!(clean(".env"), ".env");
    assert_eq!(clean("..."), "file");
    assert_eq!(
        clean(&format!("{}.png", "a".repeat(100))),
        format!("{}.png", "a".repeat(LONGEST - 4))
    );
}

#[test]
fn pastes_paths_with_spaces_in_quotes() {
    assert_eq!(
        paste(Path::new("/home/jo/Fogde/.app/files/ab12-image.png")),
        "\x1b[200~/home/jo/Fogde/.app/files/ab12-image.png\x1b[201~"
    );
    assert_eq!(
        paste(Path::new(
            r"C:\Users\Jo Doe\Fogde\.app\files\ab12-image.png"
        )),
        "\x1b[200~\"C:\\Users\\Jo Doe\\Fogde\\.app\\files\\ab12-image.png\"\x1b[201~"
    );
}
