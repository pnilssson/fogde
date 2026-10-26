use gpui_kit::{Bounds, Hsla, PathBuilder, Pixels, Window, fill, point, px, size};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

use Weight::{Double as D, Heavy as H, Light as L, None as N};

struct Arms {
    up: Weight,
    down: Weight,
    left: Weight,
    right: Weight,
}

fn lines(char: char) -> Option<Arms> {
    let (up, down, left, right) = match char {
        '─' => (N, N, L, L),
        '━' => (N, N, H, H),
        '│' => (L, L, N, N),
        '┃' => (H, H, N, N),
        '┌' => (N, L, N, L),
        '┏' => (N, H, N, H),
        '┐' => (N, L, L, N),
        '┓' => (N, H, H, N),
        '└' => (L, N, N, L),
        '┗' => (H, N, N, H),
        '┘' => (L, N, L, N),
        '┛' => (H, N, H, N),
        '├' => (L, L, N, L),
        '┣' => (H, H, N, H),
        '┤' => (L, L, L, N),
        '┫' => (H, H, H, N),
        '┬' => (N, L, L, L),
        '┳' => (N, H, H, H),
        '┴' => (L, N, L, L),
        '┻' => (H, N, H, H),
        '┼' => (L, L, L, L),
        '╋' => (H, H, H, H),
        '╴' => (N, N, L, N),
        '╵' => (L, N, N, N),
        '╶' => (N, N, N, L),
        '╷' => (N, L, N, N),
        '═' => (N, N, D, D),
        '║' => (D, D, N, N),
        '╔' => (N, D, N, D),
        '╗' => (N, D, D, N),
        '╚' => (D, N, N, D),
        '╝' => (D, N, D, N),
        '╠' => (D, D, N, D),
        '╣' => (D, D, D, N),
        '╦' => (N, D, D, D),
        '╩' => (D, N, D, D),
        '╬' => (D, D, D, D),
        _ => return None,
    };

    Some(Arms {
        up,
        down,
        left,
        right,
    })
}

fn rounded(char: char) -> Option<(f32, f32)> {
    match char {
        '╭' => Some((1., 1.)),
        '╮' => Some((-1., 1.)),
        '╯' => Some((-1., -1.)),
        '╰' => Some((1., -1.)),
        _ => None,
    }
}

type Part = (f32, f32, f32, f32, f32);

fn block(char: char) -> Option<Vec<Part>> {
    let full = 1.;
    let shade = |alpha: f32| Some(vec![(0., 0., 1., 1., alpha)]);
    let eighths = |count: f32| Some(vec![(0., 1. - count / 8., 1., count / 8., full)]);
    let left = |count: f32| Some(vec![(0., 0., count / 8., 1., full)]);
    let quads = |parts: &[(f32, f32)]| {
        Some(
            parts
                .iter()
                .map(|(x, y)| (*x, *y, 0.5, 0.5, full))
                .collect::<Vec<_>>(),
        )
    };

    match char {
        '▀' => Some(vec![(0., 0., 1., 0.5, full)]),
        '▁' => eighths(1.),
        '▂' => eighths(2.),
        '▃' => eighths(3.),
        '▄' => eighths(4.),
        '▅' => eighths(5.),
        '▆' => eighths(6.),
        '▇' => eighths(7.),
        '█' => eighths(8.),
        '▉' => left(7.),
        '▊' => left(6.),
        '▋' => left(5.),
        '▌' => left(4.),
        '▍' => left(3.),
        '▎' => left(2.),
        '▏' => left(1.),
        '▐' => Some(vec![(0.5, 0., 0.5, 1., full)]),
        '░' => shade(0.25),
        '▒' => shade(0.5),
        '▓' => shade(0.75),
        '▔' => Some(vec![(0., 0., 1., 0.125, full)]),
        '▕' => Some(vec![(0.875, 0., 0.125, 1., full)]),
        '▖' => quads(&[(0., 0.5)]),
        '▗' => quads(&[(0.5, 0.5)]),
        '▘' => quads(&[(0., 0.)]),
        '▙' => quads(&[(0., 0.), (0., 0.5), (0.5, 0.5)]),
        '▚' => quads(&[(0., 0.), (0.5, 0.5)]),
        '▛' => quads(&[(0., 0.), (0.5, 0.), (0., 0.5)]),
        '▜' => quads(&[(0., 0.), (0.5, 0.), (0.5, 0.5)]),
        '▝' => quads(&[(0.5, 0.)]),
        '▞' => quads(&[(0.5, 0.), (0., 0.5)]),
        '▟' => quads(&[(0.5, 0.), (0., 0.5), (0.5, 0.5)]),
        _ => None,
    }
}

pub fn drawn(char: char) -> bool {
    lines(char).is_some() || rounded(char).is_some() || block(char).is_some()
}

fn thickness(weight: Weight, light: Pixels) -> Pixels {
    match weight {
        Weight::Heavy => light * 2.,
        _ => light,
    }
}

pub fn paint(char: char, cell: Bounds<Pixels>, color: Hsla, window: &mut Window) {
    let light = (cell.size.width / 8.).max(px(1.)).round();
    let center = cell.center();

    if let Some(parts) = block(char) {
        for (x, y, width, height, alpha) in parts {
            let bounds = Bounds::new(
                point(
                    cell.origin.x + cell.size.width * x,
                    cell.origin.y + cell.size.height * y,
                ),
                size(cell.size.width * width, cell.size.height * height),
            );
            window.paint_quad(fill(bounds, color.opacity(alpha)));
        }
        return;
    }

    if let Some((dx, dy)) = rounded(char) {
        let radius = cell.size.width.min(cell.size.height) / 2.;
        let start = point(center.x, center.y + cell.size.height / 2. * dy);
        let end = point(center.x + cell.size.width / 2. * dx, center.y);
        let mut path = PathBuilder::stroke(light);
        path.move_to(start);
        path.line_to(point(center.x, center.y + radius * dy));
        path.curve_to(point(center.x + radius * dx, center.y), center);
        path.line_to(end);
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
        return;
    }

    let Some(arms) = lines(char) else {
        return;
    };
    let segment = |from: Pixels,
                   to: Pixels,
                   at: Pixels,
                   width: Pixels,
                   vertical: bool,
                   window: &mut Window| {
        let (low, high) = if from < to { (from, to) } else { (to, from) };
        let bounds = if vertical {
            Bounds::new(point(at - width / 2., low), size(width, high - low))
        } else {
            Bounds::new(point(low, at - width / 2.), size(high - low, width))
        };
        window.paint_quad(fill(bounds, color));
    };
    let reach = |weight: Weight| thickness(weight, light) / 2.;
    let horizontal_reach = reach(arms.up).max(reach(arms.down));
    let vertical_reach = reach(arms.left).max(reach(arms.right));

    for (weight, to, vertical) in [
        (arms.left, cell.origin.x, false),
        (arms.right, cell.origin.x + cell.size.width, false),
        (arms.up, cell.origin.y, true),
        (arms.down, cell.origin.y + cell.size.height, true),
    ] {
        let (at, from) = if vertical {
            (
                center.x,
                center.y
                    + if to < center.y {
                        vertical_reach
                    } else {
                        -vertical_reach
                    },
            )
        } else {
            (
                center.y,
                center.x
                    + if to < center.x {
                        horizontal_reach
                    } else {
                        -horizontal_reach
                    },
            )
        };
        match weight {
            Weight::None => {}
            Weight::Double => {
                for offset in [-light, light] {
                    segment(from, to, at + offset, light, vertical, window);
                }
            }
            weight => segment(from, to, at, thickness(weight, light), vertical, window),
        }
    }
}
