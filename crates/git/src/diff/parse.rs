use crate::status::Change;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub path: String,
    pub from: Option<String>,
    pub change: Change,
    pub added: u32,
    pub removed: u32,
    pub binary: bool,
    pub patch: String,
}

pub fn split(output: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut rest = output;
    while let Some(start) = rest.find("diff --git ") {
        let body = &rest[start..];
        let end = body[1..]
            .find("\ndiff --git ")
            .map_or(body.len(), |at| at + 2);
        if let Some(chunk) = chunk(&body[..end]) {
            chunks.push(chunk);
        }
        rest = &body[end..];
    }

    chunks
}

fn chunk(text: &str) -> Option<Chunk> {
    let mut lines = text.lines();
    let header = lines.next()?.strip_prefix("diff --git ")?;
    let (mut old, mut new) = (None, None);
    let mut change = Change::Modified;
    let (mut added, mut removed, mut binary, mut hunks) = (0, 0, false, false);

    for line in lines {
        if hunks {
            if line.starts_with('+') {
                added += 1;
            } else if line.starts_with('-') {
                removed += 1;
            }
            continue;
        }

        if line.starts_with("@@") {
            hunks = true;
        } else if line.starts_with("new file mode") {
            change = Change::Added;
        } else if line.starts_with("deleted file mode") {
            change = Change::Deleted;
        } else if let Some(path) = line.strip_prefix("rename from ") {
            change = Change::Renamed;
            old = Some(unquote(path));
        } else if let Some(path) = line.strip_prefix("rename to ") {
            new = Some(unquote(path));
        } else if let Some(path) = line.strip_prefix("copy from ") {
            change = Change::Copied;
            old = Some(unquote(path));
        } else if let Some(path) = line.strip_prefix("copy to ") {
            new = Some(unquote(path));
        } else if let Some(path) = line.strip_prefix("--- ") {
            old = old.or_else(|| side(path, "a/"));
        } else if let Some(path) = line.strip_prefix("+++ ") {
            new = new.or_else(|| side(path, "b/"));
        } else if line.starts_with("Binary files ") || line == "GIT binary patch" {
            binary = true;
        }
    }

    let fallback = halves(header);
    let path = new.clone().or_else(|| old.clone()).or(fallback)?;
    let from = match change {
        Change::Renamed | Change::Copied => old.filter(|old| *old != path),
        _ => None,
    };

    Some(Chunk {
        path,
        from,
        change,
        added,
        removed,
        binary,
        patch: text.to_owned(),
    })
}

fn side(path: &str, prefix: &str) -> Option<String> {
    if path == "/dev/null" {
        return None;
    }

    unquote(path.trim_end_matches('\t'))
        .strip_prefix(prefix)
        .map(str::to_owned)
}

fn halves(header: &str) -> Option<String> {
    if header.starts_with('"') {
        let end = closing(header)?;
        return unquote(&header[..=end])
            .strip_prefix("a/")
            .map(str::to_owned);
    }

    let length = header.len().checked_sub(5)? / 2;
    let path = header.get(2..2 + length)?;

    (header.get(..2) == Some("a/") && header.get(2 + length..) == Some(&format!(" b/{path}")))
        .then(|| path.to_owned())
}

fn closing(quoted: &str) -> Option<usize> {
    let mut escaped = false;
    for (index, c) in quoted.char_indices().skip(1) {
        match c {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return Some(index),
            _ => escaped = false,
        }
    }

    None
}

pub fn unquote(text: &str) -> String {
    let Some(inner) = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    else {
        return text.to_owned();
    };

    let mut bytes = Vec::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buffer = [0; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            continue;
        }

        match chars.next() {
            Some('n') => bytes.push(b'\n'),
            Some('t') => bytes.push(b'\t'),
            Some('r') => bytes.push(b'\r'),
            Some('a') => bytes.push(0x07),
            Some('b') => bytes.push(0x08),
            Some('f') => bytes.push(0x0c),
            Some('v') => bytes.push(0x0b),
            Some(digit @ '0'..='7') => {
                let mut value = digit.to_digit(8).unwrap_or(0);
                for _ in 0..2 {
                    if let Some(next) = chars.next_if(|next| next.is_digit(8)) {
                        value = value * 8 + next.to_digit(8).unwrap_or(0);
                    }
                }
                bytes.push(u8::try_from(value).unwrap_or(b'?'));
            }
            Some(other) => {
                let mut buffer = [0; 4];
                bytes.extend_from_slice(other.encode_utf8(&mut buffer).as_bytes());
            }
            None => bytes.push(b'\\'),
        }
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn untracked(name: &str, bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut written = format!(
        "diff --git a/{name} b/{name}\nnew file mode 100644\n--- /dev/null\n+++ b/{name}\n"
    );
    if lines.is_empty() {
        return written;
    }

    written.push_str("@@ -0,0 +1,");
    written.push_str(&lines.len().to_string());
    written.push_str(" @@\n");
    for line in &lines {
        written.push('+');
        written.push_str(line);
    }
    if !text.ends_with('\n') {
        written.push_str("\n\\ No newline at end of file\n");
    }

    written
}

#[cfg(test)]
mod tests;
