use protocol::terminals::{Scroll as Direction, TerminalScrolled};

use crate::num;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scroll {
    pub offset: u32,
    pub max: u32,
    pub rows: u32,
}

impl From<&TerminalScrolled> for Scroll {
    fn from(scrolled: &TerminalScrolled) -> Self {
        Self {
            offset: scrolled.offset,
            max: scrolled.max,
            rows: scrolled.rows,
        }
    }
}

impl Scroll {
    pub fn share(self) -> f32 {
        num::real(self.rows) / num::real(self.rows.saturating_add(self.max))
    }

    pub fn place(self, offset: u32) -> f32 {
        1. - num::real(offset) / num::real(self.max)
    }

    pub fn dragged(self, start: u32, delta: f32, travel: f32) -> u32 {
        if travel <= 0. {
            return start;
        }

        let offset = num::real(start) - delta / travel * num::real(self.max);
        num::round(offset).min(self.max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wheel {
    pub direction: Direction,
    pub lines: u16,
}

#[derive(Default)]
pub struct Accumulator {
    pending: f32,
}

impl Accumulator {
    pub fn add(&mut self, pixels: f32, line: f32) -> Option<Wheel> {
        if line <= 0. {
            return None;
        }

        self.pending += pixels;
        let lines = (self.pending / line).trunc();
        if lines == 0. {
            return None;
        }

        self.pending -= lines * line;
        Some(Wheel {
            direction: if lines < 0. {
                Direction::Up
            } else {
                Direction::Down
            },
            lines: num::floor_u16(lines.abs()),
        })
    }
}

pub fn cell(offset: f32, extent: f32, count: u16) -> u16 {
    if extent <= 0. || count == 0 {
        return 0;
    }

    num::floor_u16(offset / extent * f32::from(count)).min(count - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCROLL: Scroll = Scroll {
        offset: 100,
        max: 300,
        rows: 100,
    };

    #[test]
    fn the_thumb_covers_the_share_of_the_history_that_is_visible() {
        assert!((SCROLL.share() - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn the_thumb_sits_at_the_bottom_when_the_terminal_follows_the_output() {
        assert!((SCROLL.place(0) - 1.).abs() < f32::EPSILON);
        assert!(SCROLL.place(300).abs() < f32::EPSILON);
    }

    #[test]
    fn dragging_the_thumb_up_scrolls_back_in_proportion_to_the_track() {
        assert_eq!(SCROLL.dragged(0, -75., 150.), 150);
        assert_eq!(SCROLL.dragged(150, 75., 150.), 0);
    }

    #[test]
    fn a_drag_cannot_scroll_past_either_end() {
        assert_eq!(SCROLL.dragged(280, -500., 150.), 300);
        assert_eq!(SCROLL.dragged(20, 500., 150.), 0);
    }

    #[test]
    fn a_track_too_short_to_travel_leaves_the_position_alone() {
        assert_eq!(SCROLL.dragged(40, 30., 0.), 40);
    }

    #[test]
    fn wheel_pixels_add_up_to_whole_lines() {
        let mut wheel = Accumulator::default();
        assert_eq!(wheel.add(10., 20.), None);
        assert_eq!(
            wheel.add(35., 20.),
            Some(Wheel {
                direction: Direction::Down,
                lines: 2
            })
        );
        assert_eq!(
            wheel.add(-50., 20.),
            Some(Wheel {
                direction: Direction::Up,
                lines: 2
            })
        );
    }

    #[test]
    fn the_pointer_maps_to_the_cell_under_it() {
        assert_eq!(cell(0., 800., 80), 0);
        assert_eq!(cell(799., 800., 80), 79);
        assert_eq!(cell(900., 800., 80), 79);
        assert_eq!(cell(-5., 800., 80), 0);
    }
}
