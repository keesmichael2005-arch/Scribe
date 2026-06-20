use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::platform::{HotkeyBinding, HotkeyEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn register(app_handle: &tauri::AppHandle, tx: tokio::sync::mpsc::Sender<HotkeyEvent>) {
    let tx = Arc::new(tokio::sync::Mutex::new(tx));
    let pressed = Arc::new(AtomicBool::new(false));

    let tx_clone = tx.clone();
    let pressed_clone = pressed.clone();
    app_handle
        .global_shortcut()
        .on_shortcut("ctrl+alt+space", move |_app, _shortcut, event| {
            let sender = tx_clone.blocking_lock();
            match event.state {
                ShortcutState::Pressed => {
                    let was_pressed = pressed_clone.swap(true, Ordering::SeqCst);
                    if !was_pressed {
                        tracing::info!(binding = %HotkeyBinding::ChordCtrlOptSpace, event = ?HotkeyEvent::Press);
                        let _ = sender.try_send(HotkeyEvent::Press);
                    }
                }
                ShortcutState::Released => {
                    let was_pressed = pressed_clone.swap(false, Ordering::SeqCst);
                    if was_pressed {
                        tracing::info!(binding = %HotkeyBinding::ChordCtrlOptSpace, event = ?HotkeyEvent::Release);
                        let _ = sender.try_send(HotkeyEvent::Release);
                    }
                }
            }
        })
        .expect("failed to register ctrl+alt+space hotkey");
}
