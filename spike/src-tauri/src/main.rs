// Prevents a console window from appearing on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod hotkey;
mod panel;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let window = app
                .get_webview_window("spike")
                .expect("spike window not found — check tauri.conf.json label");

            // apply_panel_style MUST run before any window.show() so the
            // NonActivatingPanel style mask is in place before first display.
            panel::apply_panel_style(&window)?;

            hotkey::register_hotkeys(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application")
}
