use gpui_kit::{Hsla, Rgba};
use terminal::grid::term::cell::Flags;
use terminal::grid::term::color::Colors;
use terminal::grid::vte::ansi::{Color, NamedColor, Rgb};

use crate::theme::Tokens;

const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

pub struct Palette {
    pub ansi: [Hsla; 16],
    pub foreground: Hsla,
    pub background: Hsla,
    pub cursor: Hsla,
    pub accent: Hsla,
    pub selection: Hsla,
}

impl Palette {
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            ansi: tokens.terminal.ansi,
            foreground: tokens.foreground,
            background: tokens.surface[0],
            cursor: tokens.foreground,
            accent: tokens.surface[0],
            selection: tokens.terminal.selection,
        }
    }

    fn rgb(rgb: Rgb) -> Hsla {
        Rgba {
            r: f32::from(rgb.r) / 255.,
            g: f32::from(rgb.g) / 255.,
            b: f32::from(rgb.b) / 255.,
            a: 1.,
        }
        .into()
    }

    fn indexed(&self, index: u8, colors: &Colors) -> Hsla {
        if let Some(rgb) = colors[usize::from(index)] {
            return Self::rgb(rgb);
        }

        match index {
            0..=15 => self.ansi[usize::from(index)],
            16..=231 => {
                let cube = index - 16;
                Self::rgb(Rgb {
                    r: CUBE[usize::from(cube / 36)],
                    g: CUBE[usize::from(cube / 6 % 6)],
                    b: CUBE[usize::from(cube % 6)],
                })
            }
            _ => {
                let level = 8 + (index - 232) * 10;
                Self::rgb(Rgb {
                    r: level,
                    g: level,
                    b: level,
                })
            }
        }
    }

    fn named(&self, name: NamedColor, colors: &Colors) -> Option<Hsla> {
        let index = name as usize;
        if let Some(rgb) = colors[index] {
            return Some(Self::rgb(rgb));
        }

        match name {
            NamedColor::Foreground | NamedColor::BrightForeground => Some(self.foreground),
            NamedColor::DimForeground => Some(self.foreground.opacity(0.66)),
            NamedColor::Background => None,
            NamedColor::Cursor => Some(self.cursor),
            NamedColor::DimBlack
            | NamedColor::DimRed
            | NamedColor::DimGreen
            | NamedColor::DimYellow
            | NamedColor::DimBlue
            | NamedColor::DimMagenta
            | NamedColor::DimCyan
            | NamedColor::DimWhite => {
                let base = index - NamedColor::DimBlack as usize;
                Some(self.ansi[base].opacity(0.66))
            }
            _ => Some(self.ansi[index.min(15)]),
        }
    }

    pub fn resolve(&self, color: Color, colors: &Colors) -> Option<Hsla> {
        match color {
            Color::Named(name) => self.named(name, colors),
            Color::Spec(rgb) => Some(Self::rgb(rgb)),
            Color::Indexed(index) => Some(self.indexed(index, colors)),
        }
    }

    pub fn cell(
        &self,
        fg: Color,
        bg: Color,
        flags: Flags,
        colors: &Colors,
    ) -> (Hsla, Option<Hsla>) {
        let mut foreground = self.resolve(fg, colors).unwrap_or(self.foreground);
        let mut background = self.resolve(bg, colors);
        if flags.contains(Flags::DIM) {
            foreground = foreground.opacity(0.66);
        }
        if flags.contains(Flags::INVERSE) {
            let swapped = background.unwrap_or(self.background);
            background = Some(foreground);
            foreground = swapped;
        }
        if flags.contains(Flags::HIDDEN) {
            foreground = background.unwrap_or(self.background).opacity(0.);
        }

        (foreground, background)
    }
}
