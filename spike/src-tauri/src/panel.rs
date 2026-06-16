#[cfg(target_os = "macos")]
mod imp {
    use objc2_app_kit::NSWindowStyleMask;
    use tauri::Manager;
    use tauri_nspanel::{tauri_panel, Panel, WebviewWindowExt};

    // Gate C verdict: COMPATIBLE — wrapper crate usable.
    // SHA: a3122e894383aa068ec5365a42994e3ac94ba1b6
    // canBecomeKeyWindow and canBecomeMainWindow are overridden to false
    // so the panel never steals key-window status from the frontmost app.
    tauri_panel! {
        SpikePanel {
            config: {
                can_become_key_window: false,
                can_become_main_window: false,
            }
        }
    }

    pub fn apply(window: &tauri::WebviewWindow) -> tauri::Result<()> {
        let w = window.clone();
        window.run_on_main_thread(move || {
            let panel = w
                .to_panel::<SpikePanel>()
                .expect("NSPanel conversion failed — check tauri-nspanel SHA");
            // NSWindowStyleMaskNonActivatingPanel = 128 (0x80)
            // Prevents the panel from activating (stealing key-window) on click.
            panel.set_style_mask(NSWindowStyleMask::from_bits_retain(128));
            // NSFloatingWindowLevel = 3
            // Keeps the panel above normal windows without reaching kCGMaximumWindowLevel.
            panel.set_level(3);
        })
    }
}

pub fn apply_panel_style(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    return imp::apply(window);
    #[allow(unreachable_code)]
    Ok(())
}
