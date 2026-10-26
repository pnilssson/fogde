use gpui_kit::{BoxShadow, Hsla, point, px, rgb, rgba};

pub struct Tokens {
    pub surface: [Hsla; 8],
    pub tint: Hsla,
    pub hover: Hsla,
    pub selected: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub destructive: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub shadows: [Vec<BoxShadow>; 8],
    pub terminal: Terminal,
}

pub struct Terminal {
    pub selection: Hsla,
    pub scrollbar: Hsla,
    pub scrollbar_hover: Hsla,
    pub scrollbar_active: Hsla,
    pub ansi: [Hsla; 16],
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

fn alpha(hex: u32) -> Hsla {
    rgba(hex).into()
}

fn shadow(color: Hsla, y: f32, blur: f32, spread: f32, inset: bool) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset,
    }
}

fn drops(color: Hsla, level: usize) -> Vec<BoxShadow> {
    [1., 3., 6., 12., 24., 48., 96.]
        .into_iter()
        .take(level - 1)
        .map(|size| shadow(color, size, size, -size / 2., false))
        .collect()
}

fn light_shadows() -> [Vec<BoxShadow>; 8] {
    let color = alpha(0x0000_000f);
    std::array::from_fn(|index| {
        let mut shadows = vec![shadow(color, 0., 0., 1., false)];
        shadows.extend(drops(color, index + 1));
        shadows
    })
}

fn dark_shadows() -> [Vec<BoxShadow>; 8] {
    let highlights = [
        0x0000_0000,
        0xffff_ff03,
        0xffff_ff05,
        0xffff_ff05,
        0xffff_ff0a,
        0xffff_ff0a,
        0xffff_ff0f,
        0xffff_ff0f,
    ];
    let rings = [
        0xffff_ff05,
        0xffff_ff05,
        0xffff_ff05,
        0xffff_ff0a,
        0xffff_ff0a,
        0xffff_ff0f,
        0xffff_ff0f,
        0xffff_ff0f,
    ];
    let outlines = [
        0,
        0,
        0x0000_001f,
        0x0000_0024,
        0x0000_0029,
        0x0000_002e,
        0x0000_0033,
        0x0000_0038,
    ];
    let drop = alpha(0x0000_002e);
    std::array::from_fn(|index| {
        let mut shadows = Vec::new();
        if index > 0 {
            shadows.push(shadow(alpha(highlights[index]), 1., 0., 0., true));
        }
        shadows.push(shadow(alpha(rings[index]), 0., 0., 1., true));
        if outlines[index] != 0 {
            shadows.push(shadow(alpha(outlines[index]), 0., 0., 1., false));
        }
        shadows.extend(drops(drop, index + 1));
        shadows
    })
}

impl Tokens {
    pub fn light() -> Self {
        let raised = alpha(0xffff_ff59);
        Self {
            surface: [
                color(0x00fa_fafa),
                alpha(0xffff_ff26),
                raised,
                raised,
                raised,
                raised,
                raised,
                raised,
            ],
            tint: alpha(0xfafa_fa99),
            hover: alpha(0x0000_000a),
            selected: alpha(0x0000_0012),
            foreground: color(0x0009_090b),
            border: alpha(0x0000_0014),
            destructive: color(0x00e7_000b),
            success: color(0x0000_a63e),
            warning: color(0x00fe_9a00),
            shadows: light_shadows(),
            terminal: Terminal {
                selection: alpha(0x1818_1b24),
                scrollbar: alpha(0x1818_1b26),
                scrollbar_hover: alpha(0x1818_1b40),
                scrollbar_active: alpha(0x1818_1b59),
                ansi: [
                    0x0027_272a,
                    0x00b9_1c1c,
                    0x0015_803d,
                    0x00a1_6207,
                    0x001d_4ed8,
                    0x00a2_1caf,
                    0x000e_7490,
                    0x0052_525b,
                    0x0071_717a,
                    0x0099_1b1b,
                    0x0016_6534,
                    0x0085_4d0e,
                    0x001e_40af,
                    0x0086_198f,
                    0x0015_5e75,
                    0x0018_181b,
                ]
                .map(color),
            },
        }
    }

    pub fn dark() -> Self {
        Self {
            surface: [
                color(0x0017_1717),
                alpha(0xffff_ff08),
                alpha(0xffff_ff0f),
                alpha(0xffff_ff17),
                alpha(0xffff_ff1f),
                alpha(0xffff_ff26),
                alpha(0xffff_ff2e),
                alpha(0xffff_ff36),
            ],
            tint: alpha(0x1717_1780),
            hover: alpha(0xffff_ff0f),
            selected: alpha(0xffff_ff1a),
            foreground: color(0x00fa_fafa),
            border: alpha(0xffff_ff1a),
            destructive: color(0x00ff_6467),
            success: color(0x0000_c950),
            warning: color(0x00ff_b900),
            shadows: dark_shadows(),
            terminal: Terminal {
                selection: alpha(0xffff_ff2e),
                scrollbar: alpha(0xffff_ff26),
                scrollbar_hover: alpha(0xffff_ff40),
                scrollbar_active: alpha(0xffff_ff59),
                ansi: [
                    0x003f_3f46,
                    0x00f8_7171,
                    0x0086_efac,
                    0x00fd_e047,
                    0x0093_c5fd,
                    0x00d8_b4fe,
                    0x0067_e8f9,
                    0x00e4_e4e7,
                    0x00a1_a1aa,
                    0x00fc_a5a5,
                    0x00bb_f7d0,
                    0x00fe_f08a,
                    0x00bf_dbfe,
                    0x00e9_d5ff,
                    0x00a5_f3fc,
                    0x00fa_fafa,
                ]
                .map(color),
            },
        }
    }
}
