use tray_icon::Icon;

pub(crate) fn scoria_icon() -> Icon {
    let decoder = png::Decoder::new(std::io::Cursor::new(ICON_PNG));
    let mut reader = decoder.read_info().expect("decode icon PNG header");
    let mut buf = vec![0u8; reader.output_buffer_size().expect("icon buffer size")];
    let info = reader.next_frame(&mut buf).expect("decode icon PNG frame");

    buf.truncate(info.buffer_size());
    Icon::from_rgba(buf, info.width, info.height).expect("build tray icon")
}

const ICON_PNG: &[u8] = include_bytes!("../../../assets/scoria-32.png");
