use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use super::Saved;

pub struct Writer {
    path: PathBuf,
    last: Mutex<u64>,
}

impl Writer {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            last: Mutex::new(0),
        }
    }

    pub fn write(&self, number: u64, saved: &Saved) -> io::Result<()> {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if number < *last {
            return Ok(());
        }

        let bytes = serde_json::to_vec_pretty(saved).map_err(io::Error::other)?;
        if let Some(folder) = self.path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        std::fs::write(&temporary, bytes)?;
        std::fs::rename(&temporary, &self.path)?;
        *last = number;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(sort: &str) -> Saved {
        Saved {
            sort: Some(sort.to_owned()),
            ..Saved::default()
        }
    }

    #[test]
    fn an_older_snapshot_never_replaces_a_newer_one() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("desktop.json");
        let writer = Writer::new(path.clone());

        writer.write(2, &saved("newer")).unwrap();
        writer.write(1, &saved("older")).unwrap();

        let read: Saved = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(read.sort.as_deref(), Some("newer"));
    }
}
