use std::fmt;

pub mod macos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionKind {
    Microphone,
    Accessibility,
    InputMonitoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HotkeyBinding {
    #[serde(rename = "fn")]
    Fn,
    #[serde(rename = "ctrl+option+space")]
    ChordCtrlOptSpace,
}

impl fmt::Display for HotkeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fn => write!(f, "fn"),
            Self::ChordCtrlOptSpace => write!(f, "ctrl+option+space"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPane {
    Microphone,
    Accessibility,
    InputMonitoring,
}

pub trait Platform {
    fn permission_status(&self, kind: PermissionKind) -> PermissionStatus;
    fn open_settings_pane(&self, pane: SettingsPane);
    fn register_hotkey(
        &self,
        binding: HotkeyBinding,
    ) -> (
        tokio::sync::oneshot::Receiver<bool>,
        tokio::sync::mpsc::Receiver<HotkeyEvent>,
    );
    fn ax_check(&self) -> bool;
}
