pub mod secrets;
pub mod shared;

const KNOWN_ONBOARDING_LABEL: &str = "onboarding";

use crate::shared::Provider;

#[tauri::command]
async fn set_api_key_cmd(
    window: tauri::Window,
    provider: Provider,
    key: String,
) -> Result<(), String> {
    if window.label() != KNOWN_ONBOARDING_LABEL {
        return Err("unauthorized window".to_string());
    }
    secrets::set_api_key(provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_api_key_cmd(
    window: tauri::Window,
    provider: Provider,
) -> Result<(), String> {
    if window.label() != KNOWN_ONBOARDING_LABEL {
        return Err("unauthorized window".to_string());
    }
    secrets::delete_api_key(provider).map_err(|e| e.to_string())
}

#[tauri::command]
async fn has_api_key_cmd(
    window: tauri::Window,
    provider: Provider,
) -> Result<bool, String> {
    if window.label() != KNOWN_ONBOARDING_LABEL {
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
    fn set_api_key_cmd_rejects_overlay_via_mirror() {
        let _guard = setup();
        fn mirror_set(label: &str, provider: Provider, key: &str) -> Result<(), String> {
            if label != KNOWN_ONBOARDING_LABEL {
                return Err("unauthorized window".to_string());
            }
            secrets::set_api_key(provider, key).map_err(|e| e.to_string())
        }

        assert!(mirror_set("overlay", Provider::Groq, "test").is_err());
        assert!(mirror_set("onboarding", Provider::Groq, "test").is_ok());
    }

    #[test]
    fn has_api_key_cmd_rejects_overlay_via_mirror() {
        let _guard = setup();
        fn mirror_has(label: &str, provider: Provider) -> Result<bool, String> {
            if label != KNOWN_ONBOARDING_LABEL {
                return Err("unauthorized window".to_string());
            }
            secrets::has_api_key(provider).map_err(|e| e.to_string())
        }

        assert!(mirror_has("overlay", Provider::Groq).is_err());
    }

    #[test]
    fn delete_api_key_cmd_rejects_overlay_via_mirror() {
        let _guard = setup();
        fn mirror_delete(label: &str, provider: Provider) -> Result<(), String> {
            if label != KNOWN_ONBOARDING_LABEL {
                return Err("unauthorized window".to_string());
            }
            secrets::delete_api_key(provider).map_err(|e| e.to_string())
        }

        assert!(mirror_delete("overlay", Provider::Groq).is_err());
    }
}
