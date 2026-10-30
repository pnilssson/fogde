use super::*;

const FAILED: &str = include_str!("failed-job.log");

#[test]
fn keeps_the_end_of_the_step_that_failed() {
    let log = excerpt(FAILED);

    assert_eq!(log.step.as_deref(), Some("just check"));
    assert_eq!(log.lines.len(), KEPT);
    assert_eq!(
        log.lines.last().map(String::as_str),
        Some("Process completed with exit code 100.")
    );
    assert!(
        log.lines
            .iter()
            .any(|line| line.contains("assertion `left == right` failed"))
    );
    assert!(log.lines.iter().all(|line| !line.contains('\u{1b}')));
    assert!(log.lines.iter().all(|line| !line.starts_with("2026-")));
    assert!(
        !log.lines
            .iter()
            .any(|line| line.contains("Post job cleanup"))
    );
}

#[test]
fn strips_colours_from_the_step_command() {
    let log = excerpt(
        "2026-10-27T20:12:46.6189348Z ##[group]Run just check\n2026-10-27T20:12:46.6189645Z \u{1b}[36;1mjust check\u{1b}[0m\n2026-10-27T20:12:46.6254581Z ##[endgroup]\n2026-10-27T20:12:46.6370010Z cargo fmt --all --check\n2026-10-27T20:13:39.6710055Z ##[error]Process completed with exit code 1.\n",
    );

    assert_eq!(log.step.as_deref(), Some("just check"));
    assert_eq!(
        log.lines,
        [
            "cargo fmt --all --check",
            "Process completed with exit code 1."
        ]
    );
}

#[test]
fn keeps_the_last_lines_of_a_log_without_an_error_marker() {
    let text = (1..=100)
        .map(|n| format!("2026-10-27T20:13:39.0000000Z line {n}"))
        .collect::<Vec<_>>()
        .join("\n");

    let log = excerpt(&text);

    assert_eq!(log.step, None);
    assert_eq!(log.lines.first().map(String::as_str), Some("line 41"));
    assert_eq!(log.lines.last().map(String::as_str), Some("line 100"));
}
