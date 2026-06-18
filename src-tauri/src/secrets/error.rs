use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("keychain error")]
    Keychain(String),
    #[error("io error")]
    Io(String),
    #[error("unauthorized window")]
    Unauthorized,
}
