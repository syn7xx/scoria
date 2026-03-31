//! Build `assets/macos/Resources/scoria.icns` from `assets/macos/scoria.iconset/*.png`.
//! On Windows targets: embed manifest (Common Controls 6) and icon for `scoria.exe`.

use icns::{IconFamily, IconType, Image};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=packaging/windows/app.manifest");
    println!("cargo:rerun-if-changed=packaging/windows/scoria.ico");

    if let Err(e) = try_build_icns() {
        eprintln!("build.rs: {e}");
        process::exit(1);
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if let Err(e) = embed_windows_resources() {
            eprintln!("build.rs (windows): {e}");
            process::exit(1);
        }
    }
}

fn embed_windows_resources() -> io::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("packaging/windows/app.manifest");
    let icon = manifest_dir.join("packaging/windows/scoria.ico");
    if !manifest.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing {}", manifest.display()),
        ));
    }
    if !icon.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing {}", icon.display()),
        ));
    }

    let mut res = winres::WindowsResource::new();
    let manifest_s = manifest
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "manifest path is not UTF-8"))?;
    let icon_s = icon
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "icon path is not UTF-8"))?;
    res.set_manifest_file(manifest_s);
    res.set_icon(icon_s);
    res.compile().map_err(|e| io::Error::other(format!("{e}")))
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
