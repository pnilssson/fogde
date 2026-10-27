use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const CHUNK: u64 = 64 * 1024;

pub(super) fn last<T>(path: &Path, mut pick: impl FnMut(&str) -> Option<T>) -> Option<T> {
    let mut file = File::open(path).ok()?;
    let mut end = file.metadata().ok()?.len();
    let mut carry = Vec::new();

    while end > 0 {
        let start = end.saturating_sub(CHUNK);
        let mut chunk = vec![0; usize::try_from(end - start).ok()?];
        file.seek(SeekFrom::Start(start)).ok()?;
        file.read_exact(&mut chunk).ok()?;
        chunk.extend_from_slice(&carry);
        end = start;

        let (head, body) = if start == 0 {
            (&[][..], &chunk[..])
        } else if let Some(split) = chunk.iter().position(|&byte| byte == b'\n') {
            (&chunk[..split], &chunk[split + 1..])
        } else {
            carry = chunk;
            continue;
        };

        for line in body.rsplit(|&byte| byte == b'\n') {
            let Ok(line) = std::str::from_utf8(line) else {
                continue;
            };
            if let Some(found) = pick(line.trim_end_matches('\r')) {
                return Some(found);
            }
        }

        carry = head.to_vec();
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;

    fn lines(count: usize) -> String {
        (0..count).fold(String::new(), |mut text, index| {
            let _ = writeln!(text, "line {index:06}\r");
            text
        })
    }

    #[test]
    fn finds_the_last_match_across_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        std::fs::write(&path, lines(20_000)).unwrap();

        let found = last(&path, |line| line.ends_with('7').then(|| line.to_owned()));
        let first = last(&path, |line| (line == "line 000000").then_some(()));

        assert_eq!(found.as_deref(), Some("line 019997"));
        assert_eq!(first, Some(()));
    }

    #[test]
    fn reads_a_line_longer_than_a_chunk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let long = "x".repeat(200_000);
        std::fs::write(&path, format!("{long}\nshort\n")).unwrap();

        let found = last(&path, |line| (line.len() > 1000).then_some(line.len()));

        assert_eq!(found, Some(200_000));
    }
}
