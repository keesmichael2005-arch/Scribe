#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serde(String),
}
