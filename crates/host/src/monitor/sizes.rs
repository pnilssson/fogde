use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn measure(paths: &[PathBuf]) -> Vec<Option<u64>> {
    let skip: HashSet<&Path> = paths.iter().map(PathBuf::as_path).collect();

    paths.iter().map(|path| size(path, &skip)).collect()
}

fn size(root: &Path, skip: &HashSet<&Path>) -> Option<u64> {
    if !fs::symlink_metadata(root).ok()?.is_dir() {
        return None;
    }

    let mut total = 0;
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };

            if metadata.is_file() {
                total += metadata.len();
                continue;
            }

            let path = entry.path();
            if metadata.is_dir() && !skip.contains(path.as_path()) {
                pending.push(path);
            }
        }
    }

    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, bytes: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![0; bytes]).unwrap();
    }

    #[test]
    fn adds_up_nested_files() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        write(&app.join("README.md"), 10);
        write(&app.join("src").join("main.rs"), 20);
        write(&app.join("target").join("debug").join("app"), 300);

        assert_eq!(measure(&[app]), [Some(330)]);
    }

    #[test]
    fn leaves_nested_checkouts_to_themselves() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let login = app.join(".worktrees").join("login");
        write(&app.join("README.md"), 10);
        write(&login.join("README.md"), 40);

        assert_eq!(measure(&[app, login]), [Some(10), Some(40)]);
    }

    #[test]
    fn skips_missing_folders() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(measure(&[dir.path().join("gone")]), [None]);
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_links() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let shared = dir.path().join("shared");
        write(&app.join("README.md"), 10);
        write(&shared.join("cache.bin"), 500);
        std::os::unix::fs::symlink(&shared, app.join("cache")).unwrap();

        assert_eq!(measure(&[app]), [Some(10)]);
    }
}
