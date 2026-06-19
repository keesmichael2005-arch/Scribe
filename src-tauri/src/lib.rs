pub mod onboarding;
pub mod platform;
pub mod secrets;
pub mod shared;

use crate::shared::{Provider, ALLOWED_SECRETS_WINDOWS};
use crate::platform::macos::permissions;
use crate::platform::{PermissionStatus, HotkeyBinding, Platform};

// SECURITY (SCRIBE-3): These privileged secrets commands are dual-gated by
// onboarding.json capability entries and the runtime `window.label()` check below.
// If Sprint 3 needs the settings window to call these, add "settings" to
// ALLOWED_SECRETS_WINDOWS in shared and mirror the command permissions in settings.json.

#[tauri::command]
async fn set_api_key_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    provider: Provider,
    key: String,
) -> Result<(), String> {
    if !ALLOWED_SECRETS_WINDOWS.contains(&window.label()) {
        return Err("unauthorized window".to_string());
    }
    secrets::set_api_key(provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_api_key_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    provider: Provider,
) -> Result<(), String> {
    if !ALLOWED_SECRETS_WINDOWS.contains(&window.label()) {
        return Err("unauthorized window".to_string());
    }
    secrets::delete_api_key(provider).map_err(|e| e.to_string())
}

#[tauri::command]
async fn has_api_key_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    provider: Provider,
) -> Result<bool, String> {
    if !ALLOWED_SECRETS_WINDOWS.contains(&window.label()) {
        return Err("unauthorized window".to_string());
    }
    secrets::has_api_key(provider).map_err(|e| e.to_string())
}

#[tauri::command]
async fn request_microphone_permission_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<PermissionStatus, String> {
    if !ALLOWED_SECRETS_WINDOWS.contains(&window.label()) {
        return Err("unauthorized window".to_string());
    }
    Ok(permissions::request_microphone_permission().await)
}

#[tauri::command]
fn is_onboarding_complete_cmd() -> bool {
    onboarding::is_onboarding_complete()
}

#[tauri::command]
fn mark_onboarding_complete_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<(), String> {
    if window.label() != "onboarding" {
        return Err("unauthorized window".to_string());
    }
    onboarding::mark_onboarding_complete().map_err(|e| e.to_string())
}

#[tauri::command]
fn open_onboarding_cmd<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    onboarding::open_onboarding(&app);
}

#[tauri::command]
fn close_onboarding_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    app: tauri::AppHandle<R>,
) {
    if window.label() != "onboarding" {
        return;
    }
    onboarding::close_onboarding(&app);
}

#[tauri::command]
fn platform_microphone_status_cmd() -> PermissionStatus {
    permissions::microphone_status()
}

#[tauri::command]
fn platform_accessibility_status_cmd() -> PermissionStatus {
    permissions::accessibility_status()
}

#[tauri::command]
fn platform_open_microphone_pane_cmd() {
    crate::platform::macos::settings_link::open_microphone_pane();
}

#[tauri::command]
fn platform_open_accessibility_pane_cmd() {
    crate::platform::macos::settings_link::open_accessibility_pane();
}

#[tauri::command]
fn platform_hotkey_conflicts_cmd(binding: HotkeyBinding) -> bool {
    crate::platform::macos::MacPlatform::hotkey_conflicts(binding)
}

#[tauri::command]
fn platform_register_hotkey_cmd(app: tauri::AppHandle, binding: HotkeyBinding) {
    let platform = crate::platform::macos::MacPlatform::new(app);
    platform.register_hotkey(binding);
}

pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            set_api_key_cmd,
            delete_api_key_cmd,
            has_api_key_cmd,
            request_microphone_permission_cmd,
            is_onboarding_complete_cmd,
            mark_onboarding_complete_cmd,
            open_onboarding_cmd,
            close_onboarding_cmd,
            platform_microphone_status_cmd,
            platform_accessibility_status_cmd,
            platform_open_microphone_pane_cmd,
            platform_open_accessibility_pane_cmd,
            platform_hotkey_conflicts_cmd,
            platform_register_hotkey_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::backend::InMemoryBackend;
    use std::sync::MutexGuard;

    fn setup() -> MutexGuard<'static, ()> {
        let guard = crate::secrets::TEST_LOCK.lock().unwrap();
        crate::secrets::set_test_backend(Box::new(InMemoryBackend::new()));
        guard
    }

    #[test]
    fn scaffold_compiles() {}

    #[test]
    fn has_api_key_cmd_rejects_overlay_label() {
        let _guard = setup();
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "overlay",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let result = tauri::async_runtime::block_on(has_api_key_cmd(webview, Provider::Groq));
        assert_eq!(result, Err("unauthorized window".to_string()));
    }

    #[test]
    fn has_api_key_cmd_accepts_onboarding_label() {
        let _guard = setup();
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "onboarding",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let result = tauri::async_runtime::block_on(has_api_key_cmd(webview, Provider::Groq));
        assert!(result.is_ok());
    }

    #[test]
    fn secrets_commands_are_only_in_onboarding_capabilities() {
        let onboarding: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/onboarding.json")).unwrap();
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let onboarding_permissions = onboarding["permissions"].as_array().unwrap();
        let main_permissions = main["permissions"].as_array().unwrap();
        let secrets_commands = ["set-api-key-cmd", "delete-api-key-cmd", "has-api-key-cmd"];

        for command in secrets_commands {
            assert!(
                onboarding_permissions.contains(&serde_json::Value::String(command.to_string())),
                "onboarding capability must allow {command}"
            );
            assert!(
                !main_permissions.contains(&serde_json::Value::String(command.to_string())),
                "main capability must not allow {command}"
            );
        }
    }

    #[test]
    fn request_microphone_permission_cmd_rejects_non_onboarding_label() {
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "overlay",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let result = tauri::async_runtime::block_on(request_microphone_permission_cmd(webview));
        assert_eq!(result, Err("unauthorized window".to_string()));
    }

    #[test]
    fn privileged_platform_commands_not_in_main_capability() {
        let onboarding: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/onboarding.json")).unwrap();
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let onboarding_permissions = onboarding["permissions"].as_array().unwrap();
        let main_permissions = main["permissions"].as_array().unwrap();

        assert!(
            onboarding_permissions
                .contains(&serde_json::Value::String("request-microphone-permission-cmd".to_string())),
            "onboarding capability must allow request-microphone-permission-cmd"
        );
        assert!(
            !main_permissions
                .contains(&serde_json::Value::String("request-microphone-permission-cmd".to_string())),
            "main capability must not allow request-microphone-permission-cmd"
        );
    }

    fn temp_home() -> std::path::PathBuf {
        let tmp = std::env::temp_dir()
            .join(format!("scribe_lib_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);
        let _ = std::fs::remove_dir_all(tmp.join("Library"));
        tmp
    }

    #[test]
    fn mark_onboarding_complete_cmd_rejects_main_label() {
        let _tmp = temp_home();
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let result = mark_onboarding_complete_cmd(webview);
        assert_eq!(result, Err("unauthorized window".to_string()));
    }

    #[test]
    fn mark_onboarding_complete_cmd_accepts_onboarding_label() {
        let _tmp = temp_home();
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "onboarding",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        let result = mark_onboarding_complete_cmd(webview);
        assert!(result.is_ok());
    }

    #[test]
    fn is_onboarding_complete_cmd_returns_false_when_no_file() {
        let _tmp = temp_home();
        assert!(!is_onboarding_complete_cmd());
    }

    #[test]
    fn close_onboarding_cmd_ignores_non_onboarding_label() {
        let app = tauri::test::mock_app();
        let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        close_onboarding_cmd(webview, app.handle().clone());
    }

    #[test]
    fn new_onboarding_commands_in_capabilities() {
        let onboarding: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/onboarding.json")).unwrap();
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let onboarding_permissions = onboarding["permissions"].as_array().unwrap();
        let main_permissions = main["permissions"].as_array().unwrap();

        let onboarding_only = [
            "mark-onboarding-complete-cmd",
            "close-onboarding-cmd",
            "platform-microphone-status-cmd",
            "platform-accessibility-status-cmd",
            "platform-open-microphone-pane-cmd",
            "platform-open-accessibility-pane-cmd",
            "platform-hotkey-conflicts-cmd",
            "platform-register-hotkey-cmd",
        ];
        for cmd in onboarding_only {
            assert!(
                onboarding_permissions.contains(&serde_json::Value::String(cmd.to_string())),
                "onboarding capability must allow {cmd}"
            );
            assert!(
                !main_permissions.contains(&serde_json::Value::String(cmd.to_string())),
                "main capability must not allow {cmd}"
            );
        }

        let main_allowed = ["is-onboarding-complete-cmd", "open-onboarding-cmd"];
        for cmd in main_allowed {
            assert!(
                main_permissions.contains(&serde_json::Value::String(cmd.to_string())),
                "main capability must allow {cmd}"
            );
        }
    }
}
