use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const KEEP: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct Sizes(Mutex<HashMap<PathBuf, Measured>>);

struct Measured {
    at: Instant,
    size: Option<u64>,
    skip: BTreeSet<PathBuf>,
}

impl Sizes {
    pub fn measure(&self, paths: &[PathBuf]) -> Vec<Option<u64>> {
        let now = Instant::now();
        let stale: Vec<&PathBuf> = {
            let known = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            paths
                .iter()
                .filter(|path| {
                    known.get(*path).is_none_or(|measured| {
                        now.duration_since(measured.at) >= KEEP
                            || measured.skip != nested(path, paths)
                    })
                })
                .collect()
        };
        let skip: HashSet<&Path> = paths.iter().map(PathBuf::as_path).collect();
        let measured: Vec<(PathBuf, Measured)> = stale
            .into_iter()
            .map(|path| {
                let measured = Measured {
                    at: now,
                    size: size(path, &skip),
                    skip: nested(path, paths),
                };

                (path.clone(), measured)
            })
            .collect();

        let mut known = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        known.extend(measured);
        known.retain(|path, measured| {
            now.duration_since(measured.at) < KEEP || skip.contains(path.as_path())
        });

        paths
            .iter()
            .map(|path| known.get(path).and_then(|measured| measured.size))
            .collect()
    }
}

fn nested(root: &Path, paths: &[PathBuf]) -> BTreeSet<PathBuf> {
    paths
        .iter()
        .filter(|path| path.as_path() != root && path.starts_with(root))
        .cloned()
        .collect()
}

#[cfg(test)]
fn measure(paths: &[PathBuf]) -> Vec<Option<u64>> {
    Sizes::default().measure(paths)
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
    fn measures_a_folder_again_only_after_a_while() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let sizes = Sizes::default();
        write(&app.join("README.md"), 10);

        let first = sizes.measure(std::slice::from_ref(&app));
        write(&app.join("target").join("app"), 500);
        let again = sizes.measure(std::slice::from_ref(&app));

        assert_eq!([first, again], [[Some(10)], [Some(10)]]);
    }

    #[test]
    fn measures_again_once_a_nested_checkout_is_counted_on_its_own() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let login = app.join(".worktrees").join("login");
        let sizes = Sizes::default();
        write(&app.join("README.md"), 10);
        write(&login.join("README.md"), 40);

        let alone = sizes.measure(std::slice::from_ref(&app));
        let both = sizes.measure(&[app, login]);

        assert_eq!(alone, [Some(50)]);
        assert_eq!(both, [Some(10), Some(40)]);
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
