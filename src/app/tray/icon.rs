use tray_icon::Icon;

pub(crate) fn scoria_icon() -> anyhow::Result<Icon> {
    let decoder = png::Decoder::new(std::io::Cursor::new(ICON_PNG));
    let mut reader = decoder.read_info()?;
    let output_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("icon buffer size is unavailable"))?;
    let mut buf = vec![0u8; output_size];
    let info = reader.next_frame(&mut buf)?;

    buf.truncate(info.buffer_size());
    Icon::from_rgba(buf, info.width, info.height)
        .map_err(|e| anyhow::anyhow!("build tray icon: {e}"))
}

const ICON_PNG: &[u8] = include_bytes!("../../../assets/scoria-32.png");
