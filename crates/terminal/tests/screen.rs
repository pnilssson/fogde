use std::fmt::Write as _;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::terminals::TerminalFrame;
use terminal::{Damage, Error, Screen, width};

fn frame(cols: u16, rows: u16, full: bool, data: &str) -> TerminalFrame {
    TerminalFrame {
        id: "w1".to_owned(),
        cols,
        rows,
        full,
        data: STANDARD.encode(data),
    }
}

fn cells(rows: &[&[&str]]) -> String {
    let mut out = String::from("\x1b[?2026h\x1b[?25l\x1b]8;;\x1b\\\x1b[2J\x1b[0;39;49m");
    for (row, symbols) in rows.iter().enumerate() {
        let mut col = 0;
        let mut inline = None;
        for symbol in *symbols {
            if inline != Some(col) {
                let _ = write!(out, "\x1b[{};{}H", row + 1, col + 1);
            }
            out.push_str(symbol);
            let cells = width(symbol);
            inline = (symbol.is_ascii() && cells == 1).then_some(col + 1);
            col += cells.max(1);
        }
    }
    out.push_str("\x1b[0m\x1b[1;1H\x1b[?25h\x1b[?2026l");

    out
}

fn split(text: &str) -> Vec<&str> {
    unicode_segmentation::UnicodeSegmentation::graphemes(text, true).collect()
}

fn show(cols: u16, lines: &[&str]) -> Result<Screen, Error> {
    let rows: Vec<Vec<&str>> = lines.iter().map(|line| split(line)).collect();
    let rows: Vec<&[&str]> = rows.iter().map(Vec::as_slice).collect();
    let mut screen = Screen::new(cols, 4);
    screen.apply(&frame(
        cols,
        u16::try_from(lines.len()).unwrap_or(u16::MAX),
        true,
        &cells(&rows),
    ))?;

    Ok(screen)
}

#[test]
fn an_emoji_herdr_draws_two_cells_wide_covers_the_cell_beside_it() {
    let mut screen = Screen::new(4, 2);
    screen
        .apply(&frame(
            4,
            2,
            true,
            "\x1b[?2026h\x1b[2J\x1b[1;1Hab\x1b[?2026l",
        ))
        .unwrap();
    screen
        .apply(&frame(4, 2, false, "\x1b[?2026h\x1b[1;1H⚠️\x1b[?2026l"))
        .unwrap();

    assert_eq!(screen.text(0), "⚠️");
}

#[test]
fn a_character_too_wide_for_the_last_column_stays_on_its_row() {
    let mut screen = Screen::new(4, 2);
    screen
        .apply(&frame(
            4,
            2,
            true,
            "\x1b[?2026h\x1b[2J\x1b[1;1Habcd\x1b[2;1Hwxyz\x1b[?2026l",
        ))
        .unwrap();
    screen
        .apply(&frame(
            4,
            2,
            false,
            "\x1b[?2026h\x1b[1;4H界\x1b[1;1Hq\x1b[?2026l",
        ))
        .unwrap();

    assert_eq!(screen.text(0), "qbcd");
    assert_eq!(screen.text(1), "wxyz");
}

#[test]
fn each_grapheme_herdr_sends_keeps_its_cells() {
    let samples = [
        "👨‍👩‍👧",
        "👍🏽",
        "❤️",
        "🇸🇪",
        "⚠️",
        "界",
        "e\u{301}",
        "o\u{308}\u{301}",
        "ｶﾞ",
        "ü",
    ];

    for sample in samples {
        let screen = show(8, &[&format!("{sample}x|")]).unwrap();

        assert_eq!(screen.text(0), format!("{sample}x|"), "{sample}");
    }
}

#[test]
fn mixed_text_and_box_drawing_match_herdrs_screen() {
    let lines = [
        "family 👨‍👩‍👧 thumbs 👍🏽 heart ❤️ flag 🇸🇪 warn ⚠️",
        "cjk 日本語の文字 한국어 中文",
        "┌──────┬──────┐ ╭─╮ ╔═╗ ━━ ┃",
        "│ ░▒▓█ │ ▀▄▌▐ │ combining e\u{301} a\u{30a}",
    ];
    let screen = show(60, &lines).unwrap();

    for (row, line) in lines.iter().enumerate() {
        assert_eq!(screen.text(row), *line);
    }
}

#[test]
fn a_delta_frame_replaces_a_wide_grapheme_with_narrow_cells() {
    let mut screen = show(8, &["👍🏽x"]).unwrap();

    screen
        .apply(&frame(
            8,
            1,
            false,
            "\x1b[?2026h\x1b[1;1H\x1b[0;39;49mab\x1b[0m\x1b[?2026l",
        ))
        .unwrap();

    assert_eq!(screen.text(0), "abx");
}

#[test]
fn a_full_frame_starts_from_a_clean_screen() {
    let mut screen = show(8, &["old", "rows"]).unwrap();

    screen
        .apply(&frame(8, 2, true, "\x1b[?2026h\x1b[1;1Hnew\x1b[?2026l"))
        .unwrap();

    assert_eq!(
        (screen.text(0).as_str(), screen.text(1).as_str()),
        ("new", "")
    );
}

#[test]
fn a_frame_of_another_size_resizes_the_screen() {
    let mut screen = Screen::new(80, 24);

    screen
        .apply(&frame(4, 2, true, "\x1b[1;1Habcd\x1b[2;1Hxy"))
        .unwrap();

    assert_eq!(
        (screen.text(0).as_str(), screen.text(1).as_str()),
        ("abcd", "xy")
    );
}

#[test]
fn a_delta_frame_damages_only_the_rows_it_touches() {
    let mut screen = show(8, &["one", "two", "six"]).unwrap();

    let damage = screen
        .apply(&frame(
            8,
            3,
            false,
            "\x1b[?2026h\x1b[3;1H\x1b[0;31;49mten\x1b[0m\x1b[?2026l",
        ))
        .unwrap();

    let Damage::Rows(rows) = damage else {
        panic!("expected row damage, got {damage:?}");
    };
    assert!(rows.contains(&2));
    assert!(!rows.contains(&1));
}

#[test]
fn a_bad_frame_is_an_error() {
    let mut screen = Screen::new(4, 2);
    let mut bad = frame(4, 2, true, "");
    bad.data = "not base64!".to_owned();

    assert!(screen.apply(&bad).is_err());
}
