// Sources for retention disclosures:
//   Groq:  https://groq.com/privacy-policy/                          — verified 2026-06-18
//   OpenAI: https://openai.com/policies/enterprise-privacy/           — verified 2026-06-18
//   Groq:  https://console.groq.com/docs/legal/customer-data-processing-addendum — verified 2026-06-18

pub const APP_NAME: &str = "Scribe";
pub const KEYCHAIN_SERVICE: &str = "Scribe";
pub const ALLOWED_SECRETS_WINDOWS: &[&str] = &["onboarding"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Groq,
    OpenAI,
}

pub const PROVIDERS: &[Provider] = &[Provider::Groq, Provider::OpenAI];

pub fn egress_disclosure_copy(p: Provider) -> String {
    match p {
        Provider::Groq => "Your audio is sent to Groq for transcription. No processing happens on this device.".to_string(),
        Provider::OpenAI => "Your audio is sent to OpenAI for transcription. No processing happens on this device.".to_string(),
    }
}

// Source: https://groq.com/privacy-policy/ — verified 2026-06-18
// Groq processes audio data in-memory during transcription and does not retain it.
const RETENTION_GROQ: &str = "Groq does not retain audio data after transcription; it is processed in-memory and then discarded.";

// Source: https://openai.com/policies/enterprise-privacy/ — verified 2026-06-18
// OpenAI retains API inputs and outputs for up to 30 days for abuse monitoring.
const RETENTION_OPENAI: &str = "OpenAI retains audio data for up to 30 days for abuse monitoring, then deletes it permanently.";

pub fn retention_disclosure(p: Provider) -> &'static str {
    match p {
        Provider::Groq => RETENTION_GROQ,
        Provider::OpenAI => RETENTION_OPENAI,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn egress_copy_groq() {
        assert_eq!(super::egress_disclosure_copy(super::Provider::Groq), "Your audio is sent to Groq for transcription. No processing happens on this device.");
    }

    #[test]
    fn egress_copy_openai() {
        assert_eq!(super::egress_disclosure_copy(super::Provider::OpenAI), "Your audio is sent to OpenAI for transcription. No processing happens on this device.");
    }

    #[test]
    fn keychain_service_name() {
        assert_eq!(super::KEYCHAIN_SERVICE, "Scribe");
    }
}
