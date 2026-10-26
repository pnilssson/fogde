use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term, TermDamage};
use alacritty_terminal::vte::ansi::Processor;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use protocol::terminals::TerminalFrame;
use unicode_segmentation::UnicodeSegmentation;

use crate::grapheme;
use crate::scan::{self, Piece};

const RESET: &[u8] = b"\x1bc\x1b[?7l";
const SYNC: [&[u8]; 2] = [b"\x1b[?2026h", b"\x1b[?2026l"];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the frame is not valid base64: {0}")]
    Decode(#[from] base64::DecodeError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Damage {
    Full,
    Rows(Vec<usize>),
}

pub struct Screen {
    term: Term<VoidListener>,
    parser: Processor,
}

struct Size {
    cols: usize,
    rows: usize,
}

impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

impl Size {
    fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols: usize::from(cols.max(1)),
            rows: usize::from(rows.max(1)),
        }
    }
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        let config = Config {
            scrolling_history: 0,
            ..Config::default()
        };

        Self {
            term: Term::new(config, &Size::new(cols, rows), VoidListener),
            parser: Processor::new(),
        }
    }

    pub fn term(&self) -> &Term<VoidListener> {
        &self.term
    }

    pub fn term_mut(&mut self) -> &mut Term<VoidListener> {
        &mut self.term
    }

    pub fn apply(&mut self, frame: &TerminalFrame) -> Result<Damage, Error> {
        let data = STANDARD.decode(&frame.data)?;

        let size = Size::new(frame.cols, frame.rows);
        if size.cols != self.term.columns() || size.rows != self.term.screen_lines() {
            self.term.resize(size);
        }

        if frame.full {
            self.feed(RESET);
        }
        self.feed(&data);

        Ok(self.damage())
    }

    pub fn text(&self, row: usize) -> String {
        let grid = self.term.grid();
        if row >= grid.screen_lines() {
            return String::new();
        }

        let line = &grid[Line(i32::try_from(row).unwrap_or(i32::MAX))];
        let mut text = String::new();
        for column in 0..grid.columns() {
            let cell = &line[Column(column)];
            if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                continue;
            }
            text.push(cell.c);
            text.extend(cell.zerowidth().into_iter().flatten());
        }

        text.trim_end().to_owned()
    }

    fn feed(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let piece = scan::next(bytes);
            match piece {
                Piece::Text(text) => {
                    for grapheme in String::from_utf8_lossy(text).graphemes(true) {
                        grapheme::write(&mut self.term, grapheme);
                    }
                }
                Piece::Control(sequence) if SYNC.contains(&sequence) => {}
                Piece::Control(sequence) => self.parser.advance(&mut self.term, sequence),
            }

            let (Piece::Text(taken) | Piece::Control(taken)) = piece;
            bytes = &bytes[taken.len()..];
        }
    }

    fn damage(&mut self) -> Damage {
        let damage = match self.term.damage() {
            TermDamage::Full => Damage::Full,
            TermDamage::Partial(rows) => Damage::Rows(rows.map(|row| row.line).collect()),
        };
        self.term.reset_damage();

        damage
    }
}
