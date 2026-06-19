pub mod error;
pub mod persistence;

use tauri::Manager;

pub use error::OnboardingError;

pub const ONBOARDING_WINDOW_LABEL: &str = "onboarding";

pub fn is_onboarding_complete() -> bool {
    match persistence::read() {
        Ok(data) => data.completed,
        Err(_) => false,
    }
}

pub fn mark_onboarding_complete() -> Result<(), OnboardingError> {
    persistence::write_completed()
}

pub fn open_onboarding<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) {
    if app_handle.get_webview_window(ONBOARDING_WINDOW_LABEL).is_some() {
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app_handle,
        ONBOARDING_WINDOW_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Scribe Setup")
    .inner_size(480.0, 560.0)
    .resizable(false)
    .center()
    .build();
}

pub fn close_onboarding<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) {
    if let Some(window) = app_handle.get_webview_window(ONBOARDING_WINDOW_LABEL) {
        let _ = window.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home() -> std::path::PathBuf {
        let tmp = std::env::temp_dir()
            .join(format!("scribe_onboard_mod_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);
        let _ = std::fs::remove_dir_all(tmp.join("Library"));
        tmp
    }

    #[test]
    fn is_onboarding_complete_returns_false_when_no_file() {
        let _tmp = temp_home();
        assert!(!is_onboarding_complete());
    }

    #[test]
    fn mark_then_is_returns_true() {
        let _tmp = temp_home();
        mark_onboarding_complete().unwrap();
        assert!(is_onboarding_complete());
    }

    #[test]
    fn onboarding_window_label_is_onboarding() {
        assert_eq!(ONBOARDING_WINDOW_LABEL, "onboarding");
    }
}
