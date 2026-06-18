pub mod secrets;
pub mod shared;

use crate::shared::{Provider, ALLOWED_SECRETS_WINDOWS};

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

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_api_key_cmd,
            delete_api_key_cmd,
            has_api_key_cmd,
        ])
        // SCRIBE-3: secrets commands registered above (keychain get/set/has/delete for API keys)
        // SCRIBE-4: register platform commands (hotkey register/unregister, permissions check)
        // SCRIBE-5: register onboarding commands (open/close, is_complete, mark_complete)
        // SCRIBE-6: register tray commands (idle icon, menu items)
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
}
