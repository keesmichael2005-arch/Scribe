use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("unauthorized window")]
    Unauthorized,
}
