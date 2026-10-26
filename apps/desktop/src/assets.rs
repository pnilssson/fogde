use std::borrow::Cow;

use gpui_kit::assets::IconNamed;
use gpui_kit::{AssetSource, Result, SharedString};

macro_rules! glyphs {
    ($($name:ident => $file:literal,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Glyph {
            $($name,)*
        }

        impl IconNamed for Glyph {
            fn path(self) -> SharedString {
                match self {
                    $(Self::$name => concat!("fogde/icons/", $file, ".svg"),)*
                }
                .into()
            }
        }

        const ICONS: &[(&str, &[u8])] = &[
            $((concat!("fogde/icons/", $file, ".svg"), include_bytes!(concat!("../assets/icons/", $file, ".svg"))),)*
        ];
    };
}

glyphs! {
    ArrowCircleUp => "arrow-circle-up",
    ArrowClockwise => "arrow-clockwise",
    ArrowDown => "arrow-down",
    ArrowLeft => "arrow-left",
    ArrowRight => "arrow-right",
    ArrowUp => "arrow-up",
    Bell => "bell",
    CaretDown => "caret-down",
    CaretRight => "caret-right",
    Check => "check",
    Copy => "copy",
    Desktop => "desktop",
    DotsThree => "dots-three",
    FilePlus => "file-plus",
    FileText => "file-text",
    FolderPlus => "folder-plus",
    FolderSimplePlus => "folder-simple-plus",
    FolderSimple => "folder-simple",
    Folder => "folder",
    GearSix => "gear-six",
    GitBranch => "git-branch",
    HardDrives => "hard-drives",
    Info => "info",
    MagnifyingGlass => "magnifying-glass",
    Moon => "moon",
    Notebook => "notebook",
    PlusCircle => "plus-circle",
    Plus => "plus",
    SidebarSimpleFill => "sidebar-simple-fill",
    SidebarSimple => "sidebar-simple",
    SortAscending => "sort-ascending",
    Square => "square",
    Sun => "sun",
    TerminalWindow => "terminal-window",
    Trash => "trash",
    X => "x",
}

const BRANDS: &[(&str, &[u8])] = &[
    (
        "fogde/brands/claude-spark.svg",
        include_bytes!("../assets/brands/claude-spark.svg"),
    ),
    (
        "fogde/brands/copilot.svg",
        include_bytes!("../assets/brands/copilot.svg"),
    ),
    (
        "fogde/brands/openai.svg",
        include_bytes!("../assets/brands/openai.svg"),
    ),
];

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = ICONS.iter().chain(BRANDS).find(|(known, _)| *known == path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }

        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut found: Vec<SharedString> = ICONS
            .iter()
            .chain(BRANDS)
            .filter(|(known, _)| known.starts_with(path))
            .map(|(known, _)| SharedString::from(*known))
            .collect();
        found.extend(gpui_kit::assets::Assets.list(path)?);
        Ok(found)
    }
}
