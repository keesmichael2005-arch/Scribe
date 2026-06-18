#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

use super::error::SecretsError;

pub trait KeychainBackend: Send + Sync {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretsError>;
    fn set(&self, service: &str, account: &str, key: &str) -> Result<(), SecretsError>;
    fn delete(&self, service: &str, account: &str) -> Result<(), SecretsError>;
}

pub struct RealKeyringBackend;

impl KeychainBackend for RealKeyringBackend {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretsError> {
        let entry = keyring_core::Entry::new(service, account)
            .map_err(|e| SecretsError::Keychain(format!("{e}")))?;
        match entry.get_password() {
            Ok(pw) => Ok(Some(pw)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretsError::Keychain(format!("{e}"))),
        }
    }

    fn set(&self, service: &str, account: &str, key: &str) -> Result<(), SecretsError> {
        let entry = keyring_core::Entry::new(service, account)
            .map_err(|e| SecretsError::Keychain(format!("{e}")))?;
        entry
            .set_password(key)
            .map_err(|e| SecretsError::Keychain(format!("{e}")))
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), SecretsError> {
        let entry = keyring_core::Entry::new(service, account)
            .map_err(|e| SecretsError::Keychain(format!("{e}")))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretsError::Keychain(format!("{e}"))),
        }
    }
}

#[cfg(test)]
pub struct InMemoryBackend {
    store: Mutex<HashMap<String, HashMap<String, String>>>,
}

#[cfg(test)]
impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
impl KeychainBackend for InMemoryBackend {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, SecretsError> {
        let store = self.store.lock().unwrap();
        Ok(store
            .get(service)
            .and_then(|entries| entries.get(account))
            .cloned())
    }

    fn set(&self, service: &str, account: &str, key: &str) -> Result<(), SecretsError> {
        let mut store = self.store.lock().unwrap();
        store
            .entry(service.to_string())
            .or_default()
            .insert(account.to_string(), key.to_string());
        Ok(())
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), SecretsError> {
        let mut store = self.store.lock().unwrap();
        if let Some(entries) = store.get_mut(service) {
            entries.remove(account);
        }
        Ok(())
    }
}
