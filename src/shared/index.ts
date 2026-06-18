export const APP_NAME = "Scribe" as const;
export const KEYCHAIN_SERVICE = "Scribe" as const;

export type Provider = "groq" | "openai";

export const PROVIDERS: Provider[] = ["groq", "openai"];

export function egressDisclosureCopy(p: Provider): string {
  if (p === "groq") {
    return "Your audio is sent to Groq for transcription. No processing happens on this device.";
  }
  return "Your audio is sent to OpenAI for transcription. No processing happens on this device.";
}

// Source: https://groq.com/privacy-policy/ — verified 2026-06-18
// Groq processes audio data in-memory during transcription and does not retain it.
const RETENTION_GROQ = "Groq does not retain audio data after transcription; it is processed in-memory and then discarded.";

// Source: https://openai.com/policies/enterprise-privacy/ — verified 2026-06-18
// OpenAI retains API inputs and outputs for up to 30 days for abuse monitoring.
const RETENTION_OPENAI = "OpenAI retains audio data for up to 30 days for abuse monitoring, then deletes it permanently.";

export function retentionDisclosure(p: Provider): string {
  if (p === "groq") {
    return RETENTION_GROQ;
  }
  return RETENTION_OPENAI;
}
