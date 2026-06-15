use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

pub fn register_hotkeys(app_handle: AppHandle) {
    // ctrl+alt+space: Press/Release logging + panel toggle on Pressed.
    // Migrated from main.rs (Task 2's inline registration); the Pressed arm
    // carries the same hide/show logic Task 2 placed there.
    //
    // release_pending detects whether Released fires after each Pressed.
    // On Pressed: set true and launch a 500ms checker thread. On Released:
    // clear to false. If the checker wakes and the flag is still true, the
    // plugin did not deliver Released for this press — log the finding.
    let toggle_handle = app_handle.clone();
    let release_pending = Arc::new(AtomicBool::new(false));

    app_handle
        .global_shortcut()
        .on_shortcut("ctrl+alt+space", move |_app, _shortcut, event| {
            match event.state {
                ShortcutState::Pressed => {
                    println!("[PLUGIN] ctrl+alt+space \u{2014} Press");
                    if let Some(w) = toggle_handle.get_webview_window("spike") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                        }
                    }
                    // Arm the no-release detector. 500ms is above the ~300ms
                    // hold used in Gate B but below macOS key-repeat onset
                    // (~500ms), so a clean hold-then-release always resolves
                    // before the checker fires.
                    release_pending.store(true, Ordering::SeqCst);
                    let checker = release_pending.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(500));
                        // swap returns the old value; if still true, Released
                        // was never delivered for this press.
                        if checker.swap(false, Ordering::SeqCst) {
                            println!("[PLUGIN] ctrl+alt+space \u{2014} no Release emitted");
                        }
                    });
                }
                ShortcutState::Released => {
                    // Disarm the checker before it wakes.
                    release_pending.store(false, Ordering::SeqCst);
                    println!("[PLUGIN] ctrl+alt+space \u{2014} Release");
                }
            }
        })
        .expect("failed to register ctrl+alt+space");

    // fn: attempt plugin registration with Press/Release callback. Expected to
    // fail on most systems (fn is not a standard Quartz shortcut key); the error
    // is the finding that sends us to the rdev/IOKit path for this binding.
    match app_handle
        .global_shortcut()
        .on_shortcut("Fn", |_app, _shortcut, event| match event.state {
            ShortcutState::Pressed => println!("[PLUGIN] fn \u{2014} Press"),
            ShortcutState::Released => println!("[PLUGIN] fn \u{2014} Release"),
        }) {
        Ok(_) => println!("[PLUGIN] fn \u{2014} registered"),
        Err(e) => println!("[PLUGIN] fn \u{2014} registration failed: {}", e),
    }

    // rdev fallback thread: gated on Accessibility. AXIsProcessTrustedWithOptions
    // is called with kAXTrustedCheckOptionPrompt = false (null options dict) so
    // the system Accessibility prompt is never raised during Gate B.
    //
    // Modifier state for cross-checking ctrl+alt+space via the rdev channel.
    let ctrl_down = Arc::new(AtomicBool::new(false));
    let option_down = Arc::new(AtomicBool::new(false));
    std::thread::spawn({
        let ctrl = ctrl_down;
        let option = option_down;
        move || {
            #[cfg(target_os = "macos")]
            {
                // SAFETY: AXIsProcessTrustedWithOptions is a C function from
                // ApplicationServices.framework. Null options = no prompt.
                let trusted = unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) };
                if !trusted {
                    println!(
                        "[RDEV] Accessibility not granted \u{2014} grant in System Settings \u{2192} Privacy & Security \u{2192} Accessibility, then rebuild and relaunch"
                    );
                    return;
                }
            }
            // rdev::listen blocks; runs for the life of the process.
            let _ = rdev::listen(move |event| match event.event_type {
                rdev::EventType::KeyPress(rdev::Key::Function) => {
                    println!("[RDEV] fn \u{2014} Press")
                }
                rdev::EventType::KeyRelease(rdev::Key::Function) => {
                    println!("[RDEV] fn \u{2014} Release")
                }
                rdev::EventType::KeyPress(rdev::Key::ControlLeft)
                | rdev::EventType::KeyPress(rdev::Key::ControlRight) => {
                    ctrl.store(true, Ordering::SeqCst);
                }
                rdev::EventType::KeyRelease(rdev::Key::ControlLeft)
                | rdev::EventType::KeyRelease(rdev::Key::ControlRight) => {
                    ctrl.store(false, Ordering::SeqCst);
                }
                rdev::EventType::KeyPress(rdev::Key::Alt)
                | rdev::EventType::KeyPress(rdev::Key::AltGr) => {
                    option.store(true, Ordering::SeqCst);
                }
                rdev::EventType::KeyRelease(rdev::Key::Alt)
                | rdev::EventType::KeyRelease(rdev::Key::AltGr) => {
                    option.store(false, Ordering::SeqCst);
                }
                rdev::EventType::KeyPress(rdev::Key::Space) => {
                    if ctrl.load(Ordering::SeqCst) && option.load(Ordering::SeqCst) {
                        println!("[RDEV] ctrl+alt+space \u{2014} Press");
                    }
                }
                rdev::EventType::KeyRelease(rdev::Key::Space) => {
                    if ctrl.load(Ordering::SeqCst) && option.load(Ordering::SeqCst) {
                        println!("[RDEV] ctrl+alt+space \u{2014} Release");
                    }
                }
                _ => {}
            });
        }
    });
}
