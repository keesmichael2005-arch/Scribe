# Scribe

Scribe is a local macOS dictation app. Press a global hotkey while your cursor is
in any text field, speak, and the transcription is pasted into that field.
Single-user, local-first, no accounts.

## Status

Greenfield. This repository was reset to a fresh start; implementation begins from
the v1 blueprint. No application code yet.

## What it does (v1)

- Global hotkey activation from any focused text field, with two capture modes:
  hold-to-talk and tap-to-toggle.
- A capture pill overlay with a live waveform that never steals focus from the
  target field.
- Single-pass transcription via a swappable, OpenAI-compatible STT provider —
  Groq Whisper (default) or OpenAI `gpt-4o-transcribe`.
- Clipboard-paste delivery into the field that was focused at hotkey-press
  (multi-monitor correct), with the previous clipboard restored afterwards. No
  typing fallback.
- Local SQLite store; transcripts auto-delete after 7 days (configurable). Audio
  is ephemeral and never written to disk.
- Secrets in the macOS Keychain. A first-run onboarding wizard handles
  permissions, the provider API key, and the hotkey.

No accounts, no cloud database, and no LLM cleanup pass in v1 (cleanup and a
correction-learning layer are planned for v2).

## Stack

- **Tauri 2** — Rust core + native macOS WKWebView UI. macOS-only for v1;
  platform-specific code sits behind a trait so other platforms could be added
  later without a rewrite.
- **Frontend:** TypeScript + React.
- **Build / dev:** `cargo tauri dev` / `cargo tauri build`.

## Design docs

The full blueprint — overview, architecture, user flows, branding, rules &
constraints, security framework, and testing — lives in the OHQ wiki under
`wiki/projects/scribe/`. That wiki is the source of truth for all product and
architecture decisions; this repo implements against it.
