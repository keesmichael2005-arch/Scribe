use std::path::PathBuf;
use std::sync::Mutex;

use crate::shared::{Provider, KEYCHAIN_SERVICE};

pub mod backend;
pub mod error;

use backend::{KeychainBackend, RealKeyringBackend};
use error::SecretsError;

type BackendBox = Box<dyn KeychainBackend + Send + Sync>;

static BACKEND: Mutex<Option<BackendBox>> = Mutex::new(None);

fn with_backend<T>(
    f: impl FnOnce(&dyn KeychainBackend) -> Result<T, SecretsError>,
) -> Result<T, SecretsError> {
    let mut guard = BACKEND.lock().unwrap();
    if guard.is_none() {
        keyring::use_native_store(false)
            .map_err(|e| SecretsError::Keychain(format!("store init: {e}")))?;
        *guard = Some(Box::new(RealKeyringBackend));
    }
    f(guard.as_ref().unwrap().as_ref())
}
#[cfg(test)]
pub(crate) fn set_test_backend(backend: Box<dyn KeychainBackend + Send + Sync>) {
    let mut guard = BACKEND.lock().unwrap();
    *guard = Some(backend);
}

#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

fn provider_account(provider: Provider) -> &'static str {
    match provider {
        Provider::Groq => "scribe_api_key_groq",
        Provider::OpenAI => "scribe_api_key_openai",
    }
}

pub fn get_api_key(provider: Provider) -> Result<Option<String>, SecretsError> {
    with_backend(|b| b.get(KEYCHAIN_SERVICE, provider_account(provider)))
}

pub fn set_api_key(provider: Provider, key: &str) -> Result<(), SecretsError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(SecretsError::Keychain("empty key".to_string()));
    }
    with_backend(|b| b.set(KEYCHAIN_SERVICE, provider_account(provider), trimmed))
}

pub fn has_api_key(provider: Provider) -> Result<bool, SecretsError> {
    with_backend(|b| {
        b.get(KEYCHAIN_SERVICE, provider_account(provider))
            .map(|val| val.map_or(false, |s| !s.is_empty()))
    })
}

pub fn delete_api_key(provider: Provider) -> Result<(), SecretsError> {
    with_backend(|b| b.delete(KEYCHAIN_SERVICE, provider_account(provider)))
}

pub fn app_data_dir() -> Result<PathBuf, SecretsError> {
    let dir = base_data_dir()
        .ok_or_else(|| SecretsError::Io("could not resolve data directory".to_string()))?
        .join("Scribe");
    std::fs::create_dir_all(&dir).map_err(|e| SecretsError::Io(format!("{e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&dir).map_err(|e| SecretsError::Io(format!("{e}")))?;
        let mode = metadata.permissions().mode();
        if mode & 0o777 != 0o700 {
            let mut perms = metadata.permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&dir, perms).map_err(|e| SecretsError::Io(format!("{e}")))?;
        }
    }

    Ok(dir)
}

#[cfg(not(test))]
fn base_data_dir() -> Option<PathBuf> {
    dirs::data_dir()
}

#[cfg(test)]
fn base_data_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::Provider;
    use backend::InMemoryBackend;
    use std::sync::MutexGuard;

    fn setup() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        set_test_backend(Box::new(InMemoryBackend::new()));
        guard
    }

    #[test]
    fn round_trip_set_get() {
        let _guard = setup();
        let result = set_api_key(Provider::Groq, "test-key-123");
        assert!(result.is_ok());
        let stored = get_api_key(Provider::Groq).unwrap();
        assert_eq!(stored, Some("test-key-123".to_string()));
    }

    #[test]
    fn set_with_whitespace_trims() {
        let _guard = setup();
        set_api_key(Provider::Groq, "  sk-test  ").unwrap();
        let stored = get_api_key(Provider::Groq).unwrap();
        assert_eq!(stored, Some("sk-test".to_string()));
    }

    #[test]
    fn delete_removes_key() {
        let _guard = setup();
        set_api_key(Provider::OpenAI, "openai-key").unwrap();
        assert!(has_api_key(Provider::OpenAI).unwrap());
        delete_api_key(Provider::OpenAI).unwrap();
        assert!(!has_api_key(Provider::OpenAI).unwrap());
        assert_eq!(get_api_key(Provider::OpenAI).unwrap(), None);
    }

    #[test]
    fn has_api_key_returns_false_when_missing() {
        let _guard = setup();
        assert!(!has_api_key(Provider::Groq).unwrap());
        assert!(!has_api_key(Provider::OpenAI).unwrap());
    }

    #[test]
    fn has_api_key_returns_true_after_set() {
        let _guard = setup();
        set_api_key(Provider::Groq, "key").unwrap();
        assert!(has_api_key(Provider::Groq).unwrap());
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let _guard = setup();
        let result = delete_api_key(Provider::Groq);
        assert!(result.is_ok());
    }

    #[test]
    fn empty_key_rejected() {
        let _guard = setup();
        let result = set_api_key(Provider::Groq, "");
        assert!(result.is_err());
        match result {
            Err(SecretsError::Keychain(ref msg)) => assert!(msg.contains("empty")),
            _ => panic!("expected Keychain error"),
        }
    }

    #[test]
    fn whitespace_only_key_rejected() {
        let _guard = setup();
        let result = set_api_key(Provider::Groq, "   ");
        assert!(result.is_err());
    }

    #[test]
    fn secrets_error_display_never_contains_fake_key() {
        let fake_key = "gsk_test_NEVERAPPEAR";
        let variants: Vec<SecretsError> = vec![
            SecretsError::Keychain(fake_key.to_string()),
            SecretsError::Io(fake_key.to_string()),
            SecretsError::Unauthorized,
        ];
        for variant in variants {
            let display = variant.to_string();
            assert!(
                !display.contains(fake_key),
                "Display output for variant contained the fake key: {display}"
            );
        }
    }

    #[test]
    fn app_data_dir_creates_and_has_mode_700() {
        let tmp = std::env::temp_dir().join(format!("scribe_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);

        let dir = app_data_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.ends_with("Scribe"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&dir).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(
                mode & 0o777,
                0o700,
                "expected mode 0o700, got 0o{:o}",
                mode & 0o777
            );
        }

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn provider_isolation_groq_and_openai_separate() {
        let _guard = setup();
        set_api_key(Provider::Groq, "groq-key").unwrap();
        set_api_key(Provider::OpenAI, "openai-key").unwrap();
        assert_eq!(
            get_api_key(Provider::Groq).unwrap(),
            Some("groq-key".to_string())
        );
        assert_eq!(
            get_api_key(Provider::OpenAI).unwrap(),
            Some("openai-key".to_string())
        );
        delete_api_key(Provider::Groq).unwrap();
        assert_eq!(get_api_key(Provider::Groq).unwrap(), None);
        assert_eq!(
            get_api_key(Provider::OpenAI).unwrap(),
            Some("openai-key".to_string())
        );
    }
}
