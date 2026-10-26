mod tokens;

use std::borrow::Cow;

use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};
use gpui_kit::{App, FontWeight, Global, Styled, px};
use protocol::theme::{Material, Theme as Choice};

pub use tokens::Tokens;

pub const SANS: &str = "Manrope";
pub const MONO: &str = "JetBrains Mono";

const FONTS: [&[u8]; 7] = [
    include_bytes!("../../assets/fonts/Manrope-Regular.ttf"),
    include_bytes!("../../assets/fonts/Manrope-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/Manrope-Bold.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
];

const THEMES: &str = include_str!("../../assets/themes/fogde.json");

struct Looks {
    light: Tokens,
    dark: Tokens,
    choice: Choice,
    material: Material,
    translucent: bool,
}

impl Global for Looks {}

pub fn init(translucent: bool, material: Material, cx: &mut App) {
    if let Err(error) = cx
        .text_system()
        .add_fonts(FONTS.iter().map(|font| Cow::Borrowed(*font)).collect())
    {
        tracing::warn!(%error, "could not load the fonts");
    }

    if let Err(error) = ThemeRegistry::global_mut(cx).load_themes_from_str(THEMES) {
        tracing::warn!(%error, "could not load the theme");
    }
    let themes = ThemeRegistry::global(cx).themes();
    let light = themes.get("Fogde Light").cloned();
    let dark = themes.get("Fogde Dark").cloned();
    Theme::update(cx, |theme| {
        if let Some(light) = light {
            theme.light_theme = light;
        }
        if let Some(dark) = dark {
            theme.dark_theme = dark;
        }
    });

    cx.set_global(Looks {
        light: Tokens::light(),
        dark: Tokens::dark(),
        choice: Choice::System,
        material,
        translucent,
    });
    apply(cx);
}

pub fn tokens(cx: &App) -> &Tokens {
    let looks = cx.global::<Looks>();
    if Theme::global(cx).is_dark() {
        &looks.dark
    } else {
        &looks.light
    }
}

pub fn choice(cx: &App) -> Choice {
    cx.global::<Looks>().choice
}

pub fn material(cx: &App) -> Material {
    cx.global::<Looks>().material
}

pub fn can_be_translucent(cx: &App) -> bool {
    cx.global::<Looks>().translucent
}

pub fn translucent(cx: &App) -> bool {
    let looks = cx.global::<Looks>();
    looks.translucent && looks.material == Material::Translucent
}

pub fn choose(choice: Choice, cx: &mut App) {
    cx.global_mut::<Looks>().choice = choice;
    apply(cx);
}

pub fn choose_material(material: Material, cx: &mut App) {
    cx.global_mut::<Looks>().material = material;
    apply(cx);
}

pub fn apply(cx: &mut App) {
    let mode = match choice(cx) {
        Choice::System => ThemeMode::from(cx.window_appearance()),
        Choice::Light => ThemeMode::Light,
        Choice::Dark => ThemeMode::Dark,
    };
    Theme::change(mode, None, cx);

    if translucent(cx) {
        let tint = tokens(cx).tint;
        Theme::update(cx, |theme| theme.background = tint);
    }
}

pub trait Type: Styled + Sized {
    fn type_display(self) -> Self {
        self.text_size(px(28.))
            .line_height(px(34.))
            .font_weight(FontWeight::BOLD)
    }

    fn type_title(self) -> Self {
        self.text_size(px(14.))
            .line_height(px(20.))
            .font_weight(FontWeight(550.))
    }

    fn type_body(self) -> Self {
        self.text_size(px(13.))
            .line_height(px(20.))
            .font_weight(FontWeight::NORMAL)
    }

    fn type_caption(self) -> Self {
        self.text_size(px(12.))
            .line_height(px(16.))
            .font_weight(FontWeight::NORMAL)
    }

    fn type_code(self) -> Self {
        self.font_family(MONO)
            .text_size(px(13.))
            .line_height(px(20.))
            .font_weight(FontWeight::NORMAL)
    }
}

impl<T: Styled> Type for T {}
