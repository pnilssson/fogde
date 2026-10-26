use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

fn render(tree: &Tree, size: u16) -> anyhow::Result<Vec<u8>> {
    let mut pixmap =
        Pixmap::new(u32::from(size), u32::from(size)).context("the icon size is zero")?;
    let scale = f32::from(size) / tree.size().width().max(tree.size().height());
    resvg::render(
        tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}

fn load(path: &Path) -> anyhow::Result<Tree> {
    let data = std::fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
    Ok(Tree::from_data(&data, &Options::default())?)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [full, macos, out] = args.as_slice() else {
        bail!("usage: icons <full-bleed.svg> <macos.svg> <folder>");
    };
    std::fs::create_dir_all(out)?;
    let (full, macos) = (load(full)?, load(macos)?);

    for (name, size) in [
        ("32x32.png", 32),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
    ] {
        std::fs::write(out.join(name), render(&full, size)?)?;
    }

    let mut ico = ico::IconDir::new(ico::ResourceType::Icon);
    for size in [16, 24, 32, 48, 64, 256] {
        let image = ico::IconImage::read_png(std::io::Cursor::new(render(&full, size)?))?;
        ico.add_entry(ico::IconDirEntry::encode(&image)?);
    }
    ico.write(BufWriter::new(File::create(out.join("icon.ico"))?))?;

    let mut icns = icns::IconFamily::new();
    for size in [16, 32, 64, 128, 256, 512, 1024] {
        let image = icns::Image::read_png(std::io::Cursor::new(render(&macos, size)?))?;
        icns.add_icon(&image)?;
    }
    icns.write(BufWriter::new(File::create(out.join("icon.icns"))?))?;

    Ok(())
}
