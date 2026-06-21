use std::process::Command;

const ACCESSIBILITY_PANE_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
const MICROPHONE_PANE_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone";
const INPUT_MONITORING_PANE_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent";

pub fn open_accessibility_pane() {
    let _ = Command::new("open").arg(ACCESSIBILITY_PANE_URL).spawn();
}

pub fn open_microphone_pane() {
    let _ = Command::new("open").arg(MICROPHONE_PANE_URL).spawn();
}

pub fn open_input_monitoring_pane() {
    let _ = Command::new("open").arg(INPUT_MONITORING_PANE_URL).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessibility_pane_url_exact() {
        assert_eq!(
            ACCESSIBILITY_PANE_URL,
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
        );
    }

    #[test]
    fn microphone_pane_url_exact() {
        assert_eq!(
            MICROPHONE_PANE_URL,
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
        );
    }
}
