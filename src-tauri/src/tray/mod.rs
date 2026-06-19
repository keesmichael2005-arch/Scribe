use tauri::{
    image::Image,
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Runtime,
};

pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let icon_bytes = include_bytes!("../../icons/tray-idle-template.png");
    let img = image::load_from_memory(icon_bytes)
        .map_err(|e| {
            tauri::Error::InvalidIcon(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?
        .into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    TrayIconBuilder::with_id("scribe-idle")
        .icon(Image::new_owned(rgba, width, height))
        .icon_as_template(true)
        .build(app)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tray_init_icon_bytes_load() {
        let bytes = include_bytes!("../../icons/tray-idle-template.png");
        let img = image::load_from_memory(bytes);
        assert!(img.is_ok(), "tray icon PNG must be loadable");
    }
}
