pub mod error;
mod persistence;

pub use error::OnboardingError;

pub const ONBOARDING_WINDOW_LABEL: &str = "onboarding";

pub fn is_onboarding_complete() -> bool {
    persistence::read_onboarding()
        .map(|s| s.completed)
        .unwrap_or(false)
}

pub fn mark_onboarding_complete() -> Result<(), OnboardingError> {
    persistence::write_onboarding(true)
}

pub fn open_onboarding(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(ONBOARDING_WINDOW_LABEL) {
        let _ = window.show();
    }
}

pub fn close_onboarding(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(ONBOARDING_WINDOW_LABEL) {
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
