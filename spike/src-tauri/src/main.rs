// Prevents a console window from appearing on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod panel;

use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let window = app
                .get_webview_window("spike")
                .expect("spike window not found — check tauri.conf.json label");

            // apply_panel_style MUST run before any window.show() so the
            // NonActivatingPanel style mask is in place before first display.
            panel::apply_panel_style(&window)?;

            let app_handle = app.handle().clone();

            // ctrl+alt+space toggles the hidden panel visible/hidden.
            // "alt" (not "option") is the plugin's modifier name.
            app.handle()
                .global_shortcut()
                .on_shortcut("ctrl+alt+space", move |_app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    if let Some(w) = app_handle.get_webview_window("spike") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                        }
                    }
                })
                .expect("failed to register Ctrl+Alt+Space");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application")
}
