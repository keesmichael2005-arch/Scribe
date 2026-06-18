pub mod hotkey_fn;
pub mod hotkey_plugin;
pub mod permissions;
pub mod settings_link;

use crate::platform::{
    HotkeyBinding, HotkeyEvent, PermissionKind, PermissionStatus, Platform, SettingsPane,
};

#[derive(Clone)]
pub struct MacPlatform {
    app_handle: tauri::AppHandle,
}

impl MacPlatform {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    // Sprint 0 Gate B: fn is invisible to Quartz / CGEventTap / tauri-plugin-global-shortcut
    // / rdev event streams on Apple Silicon. A standard system shortcut query cannot find fn.
    pub fn hotkey_conflicts(binding: HotkeyBinding) -> bool {
        match binding {
            HotkeyBinding::Fn => false,
            // Ctrl+Option+Space is not a default macOS system shortcut.
            // Future: query the system shortcut registry for arbitrary bindings.
            HotkeyBinding::ChordCtrlOptSpace => false,
        }
    }

    pub fn ax_check_with_prompt() -> bool {
        #[cfg(target_os = "macos")]
        unsafe {
            use objc2::msg_send;
            use objc2::rc::Retained;
            use objc2_foundation::{NSDictionary, NSNumber, NSString};

            let key = NSString::from_str("AXTrustedCheckOptionPrompt");
            let val = NSNumber::new_bool(true);
            let cls = objc2::class!(NSDictionary);
            let dict: Retained<NSDictionary> = msg_send![
                cls,
                dictionaryWithObject: &*val,
                forKey: &*key
            ];
            extern "C" {
                fn AXIsProcessTrustedWithOptions(
                    options: *const objc2::runtime::AnyObject,
                ) -> bool;
            }
            AXIsProcessTrustedWithOptions(
                &*dict as *const NSDictionary as *const objc2::runtime::AnyObject,
            )
        }
        #[cfg(not(target_os = "macos"))]
        false
    }
}

impl Platform for MacPlatform {
    fn permission_status(&self, kind: PermissionKind) -> PermissionStatus {
        match kind {
            PermissionKind::Microphone => permissions::microphone_status(),
            PermissionKind::Accessibility => permissions::accessibility_status(),
        }
    }

    fn open_settings_pane(&self, pane: SettingsPane) {
        match pane {
            SettingsPane::Microphone => settings_link::open_microphone_pane(),
            SettingsPane::Accessibility => settings_link::open_accessibility_pane(),
        }
    }

    fn register_hotkey(
        &self,
        binding: HotkeyBinding,
    ) -> tokio::sync::mpsc::Receiver<HotkeyEvent> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        match binding {
            HotkeyBinding::Fn => {
                if let Err(e) = hotkey_fn::start(tx) {
                    tracing::error!(?e, "failed to start fn hotkey tap");
                }
            }
            HotkeyBinding::ChordCtrlOptSpace => {
                hotkey_plugin::register(&self.app_handle, tx);
            }
        }
        rx
    }

    fn ax_check(&self) -> bool {
        permissions::accessibility_status() == PermissionStatus::Granted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{HotkeyBinding, HotkeyEvent};

    #[test]
    fn hotkey_conflicts_fn_returns_false() {
        assert!(!MacPlatform::hotkey_conflicts(HotkeyBinding::Fn));
    }

    #[test]
    fn hotkey_conflicts_chord_returns_false() {
        assert!(!MacPlatform::hotkey_conflicts(HotkeyBinding::ChordCtrlOptSpace));
    }

    #[test]
    fn hotkey_binding_display_fn() {
        assert_eq!(HotkeyBinding::Fn.to_string(), "fn");
    }

    #[test]
    fn hotkey_binding_display_chord() {
        assert_eq!(
            HotkeyBinding::ChordCtrlOptSpace.to_string(),
            "ctrl+option+space"
        );
    }

    #[test]
    fn hotkey_event_press_release_are_distinct() {
        assert_ne!(HotkeyEvent::Press, HotkeyEvent::Release);
    }
}
