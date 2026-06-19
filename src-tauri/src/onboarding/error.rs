#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("io error: {0}")]
    Io(String),

    #[error("serde error: {0}")]
    Serde(String),
}

impl From<std::io::Error> for OnboardingError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for OnboardingError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}
