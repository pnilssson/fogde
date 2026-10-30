use protocol::github::Log;
use protocol::repos::Repo;

use crate::Error;
use crate::api::{checked, path};
use crate::gh::Gh;

const KEPT: usize = 60;
const RUN: &str = "##[group]Run ";

impl Gh {
    pub async fn log(&self, repo: &Repo, job: u64) -> Result<Log, Error> {
        let host = checked(repo)?;
        let text = self
            .read(&host, &path(repo, &format!("actions/jobs/{job}/logs")))
            .await?;

        Ok(excerpt(&String::from_utf8_lossy(&text)))
    }
}

fn excerpt(text: &str) -> Log {
    let lines: Vec<String> = text.lines().map(clean).collect();
    let end = lines
        .iter()
        .rposition(|line| line.starts_with("##[error]"))
        .map_or(lines.len(), |at| at + 1);
    let group = lines[..end].iter().rposition(|line| line.starts_with(RUN));
    let start = group.map_or(0, |at| {
        lines[at..end]
            .iter()
            .position(|line| line == "##[endgroup]")
            .map_or(at + 1, |offset| at + offset + 1)
    });
    let body: Vec<String> = lines[start..end]
        .iter()
        .filter(|line| !line.starts_with("##[group]") && *line != "##[endgroup]")
        .map(|line| {
            line.strip_prefix("##[error]")
                .or_else(|| line.strip_prefix("##[warning]"))
                .unwrap_or(line)
                .to_owned()
        })
        .collect();

    Log {
        step: group.map(|at| lines[at][RUN.len()..].to_owned()),
        lines: body[body.len().saturating_sub(KEPT)..].to_vec(),
    }
}

fn clean(line: &str) -> String {
    let line = line.trim_start_matches('\u{feff}');
    let line = match line.split_once(' ') {
        Some((stamp, rest)) if stamped(stamp) => rest,
        _ => line,
    };

    let mut plain = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            plain.push(c);
            continue;
        }

        if chars.next() == Some('[') {
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        }
    }

    plain.trim_end().to_owned()
}

fn stamped(word: &str) -> bool {
    word.len() >= 20
        && word.ends_with('Z')
        && word.as_bytes().get(4) == Some(&b'-')
        && word.as_bytes().get(10) == Some(&b'T')
        && word.starts_with(|c: char| c.is_ascii_digit())
}

#[cfg(test)]
mod tests;
