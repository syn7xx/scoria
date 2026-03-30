//! Build `assets/macos/Resources/scoria.icns` from `assets/macos/scoria.iconset/*.png`.

use icns::{IconFamily, IconType, Image};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if let Err(e) = try_build_icns() {
        eprintln!("build.rs: {e}");
        process::exit(1);
    }
}

fn try_build_icns() -> io::Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let iconset = manifest.join("assets/macos/scoria.iconset");
    let out = manifest.join("assets/macos/Resources/scoria.icns");

    if !iconset.is_dir() {
        return Ok(());
    }

    // Order and types must match Apple iconset filenames / pixel sizes.
    let entries: &[(&str, IconType)] = &[
        ("icon_16x16.png", IconType::RGBA32_16x16),
        ("icon_16x16@2x.png", IconType::RGBA32_16x16_2x),
        ("icon_32x32.png", IconType::RGBA32_32x32),
        ("icon_32x32@2x.png", IconType::RGBA32_32x32_2x),
        ("icon_128x128.png", IconType::RGBA32_128x128),
        ("icon_128x128@2x.png", IconType::RGBA32_128x128_2x),
        ("icon_256x256.png", IconType::RGBA32_256x256),
        ("icon_256x256@2x.png", IconType::RGBA32_256x256_2x),
        ("icon_512x512.png", IconType::RGBA32_512x512),
        ("icon_512x512@2x.png", IconType::RGBA32_512x512_2x),
    ];

    let mut family = IconFamily::new();
    for (name, ty) in entries {
        let path = iconset.join(name);
        let file = BufReader::new(File::open(&path)?);
        let image = Image::read_png(file)?;
        family.add_icon_with_type(&image, *ty)?;
    }

    std::fs::create_dir_all(out.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Resources path has no parent")
    })?)?;
    let f = File::create(&out)?;
    family.write(BufWriter::new(f))?;

    for (name, _) in entries {
        println!("cargo:rerun-if-changed={}", iconset.join(name).display());
    }

    Ok(())
}
