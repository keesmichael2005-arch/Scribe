use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde::{Deserialize, Serialize};

use crate::secrets;

use super::error::OnboardingError;

const ONBOARDING_FILE: &str = "onboarding.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingState {
    pub completed: bool,
    pub completed_at: String,
}

fn onboarding_path() -> Result<std::path::PathBuf, OnboardingError> {
    secrets::app_data_dir()
        .map(|dir| dir.join(ONBOARDING_FILE))
        .map_err(|e| OnboardingError::Io(e.to_string()))
}

pub fn read() -> Result<OnboardingState, OnboardingError> {
    let path = onboarding_path()?;
    let bytes = std::fs::read(&path)?;
    serde_json::from_slice::<OnboardingState>(&bytes).map_err(Into::into)
}

pub fn write_completed() -> Result<(), OnboardingError> {
    let path = onboarding_path()?;
    let state = OnboardingState {
        completed: true,
        completed_at: utc_now_rfc3339(),
    };
    let json =
        serde_json::to_vec_pretty(&state).map_err(|e| OnboardingError::Serde(e.to_string()))?;

    let mut f = std::fs::File::create(&path)
        .map_err(|e| OnboardingError::Io(format!("create {}: {e}", path.display())))?;
    f.write_all(&json)
        .map_err(|e| OnboardingError::Io(format!("write {}: {e}", path.display())))?;

    #[cfg(unix)]
    {
        let mut perms = f
            .metadata()
            .map_err(|e| OnboardingError::Io(e.to_string()))?
            .permissions();
        perms.set_mode(0o600);
        f.set_permissions(perms)
            .map_err(|e| OnboardingError::Io(e.to_string()))?;
    }

    Ok(())
}

fn utc_now_rfc3339() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    format_unix_utc(secs)
}

fn format_unix_utc(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    let (y, mo, d) = civil_from_days(days);

    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home() -> std::path::PathBuf {
        let tmp =
            std::env::temp_dir().join(format!("scribe_onboard_persist_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);
        // Ensure app_data_dir creates its Scribe subdirectory fresh
        let _ = std::fs::remove_dir_all(tmp.join("Library"));
        tmp
    }

    #[test]
    fn write_then_read_round_trip() {
        let _tmp = temp_home();
        write_completed().unwrap();
        let state = read().unwrap();
        assert!(state.completed);
        assert!(!state.completed_at.is_empty());
        assert!(state.completed_at.ends_with('Z'));
    }

    #[test]
    fn read_missing_returns_err() {
        let _tmp = temp_home();
        assert!(read().is_err());
    }

    #[test]
    fn file_mode_is_0600() {
        let _tmp = temp_home();
        write_completed().unwrap();
        let path = onboarding_path().unwrap();
        #[cfg(unix)]
        {
            let metadata = std::fs::metadata(&path).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "expected mode 0o600, got 0o{:o}",
                mode & 0o777
            );
        }
    }

    #[test]
    fn is_onboarding_complete_false_then_true() {
        let _tmp = temp_home();
        assert!(!super::super::is_onboarding_complete());
        super::super::mark_onboarding_complete().unwrap();
        assert!(super::super::is_onboarding_complete());
    }
}
