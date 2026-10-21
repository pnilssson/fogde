use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::terminals::{IMAGE_LIMIT, Image};
use tokio::fs;

const KEPT: Duration = Duration::from_hours(7 * 24);

pub async fn save(dir: &Path, image: Image, data: String) -> io::Result<PathBuf> {
    if data.len() > IMAGE_LIMIT.div_ceil(3) * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("the image is larger than {} MB", IMAGE_LIMIT / 1024 / 1024),
        ));
    }
    let bytes = tokio::task::spawn_blocking(move || STANDARD.decode(data))
        .await
        .map_err(io::Error::other)?
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    create(dir).await?;
    prune(dir).await;
    let name = &crate::token::generate()?[..16];
    let path = dir.join(format!("{name}.{}", extension(image)));
    fs::write(&path, bytes).await?;

    Ok(path)
}

pub fn paste(path: &Path) -> String {
    let path = path.display().to_string();
    let path = if path.contains(char::is_whitespace) {
        format!("\"{path}\"")
    } else {
        path
    };

    format!("\x1b[200~{path}\x1b[201~")
}

async fn create(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);

    builder.create(dir).await
}

async fn prune(dir: &Path) {
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return;
    };
    let Some(cutoff) = SystemTime::now().checked_sub(KEPT) else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let old = entry
            .metadata()
            .await
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| modified < cutoff);
        if old {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

fn extension(image: Image) -> &'static str {
    match image {
        Image::Png => "png",
        Image::Jpeg => "jpg",
        Image::Gif => "gif",
        Image::Webp => "webp",
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[tokio::test]
    async fn saves_the_image_and_removes_week_old_ones() {
        let dir = tempfile::tempdir().unwrap();
        let images = dir.path().join("images");
        std::fs::create_dir(&images).unwrap();
        let old = images.join("old.png");
        let recent = images.join("recent.png");
        File::create(&old)
            .unwrap()
            .set_modified(SystemTime::now() - KEPT - Duration::from_secs(60))
            .unwrap();
        File::create(&recent).unwrap();

        let saved = save(&images, Image::Jpeg, STANDARD.encode(b"jpeg"))
            .await
            .unwrap();

        assert_eq!(saved.extension().unwrap(), "jpg");
        assert_eq!(std::fs::read(&saved).unwrap(), b"jpeg");
        assert!(!old.exists());
        assert!(recent.exists());
    }

    #[tokio::test]
    async fn refuses_images_over_the_limit_and_bad_data() {
        let dir = tempfile::tempdir().unwrap();
        let images = dir.path().join("images");
        let large = "A".repeat(IMAGE_LIMIT.div_ceil(3) * 4 + 4);

        assert_eq!(
            save(&images, Image::Png, large).await.unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            save(&images, Image::Png, "not base64!".to_owned())
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert!(!images.exists());
    }

    #[test]
    fn pastes_paths_with_spaces_in_quotes() {
        assert_eq!(
            paste(Path::new("/home/jo/Fogde/.app/images/ab12.png")),
            "\x1b[200~/home/jo/Fogde/.app/images/ab12.png\x1b[201~"
        );
        assert_eq!(
            paste(Path::new(r"C:\Users\Jo Doe\Fogde\.app\images\ab12.png")),
            "\x1b[200~\"C:\\Users\\Jo Doe\\Fogde\\.app\\images\\ab12.png\"\x1b[201~"
        );
    }
}
