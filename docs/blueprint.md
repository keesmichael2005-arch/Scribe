# Product blueprint — Scribe

## North star

Kees presses a key in any focused text field on his Mac, speaks, and the exact words land where the cursor is — instantly and privately. Twelve months out we've won when he never reaches for the keyboard to enter voice content in any field on his Mac, his New Zealand accent and his personal vocabulary (agent and product names, technical terms) are transcribed accurately enough that he stops correcting obvious errors, and the clipboard-paste-with-restore cycle is trustworthy enough for sensitive input (passwords, private messages). V1 advances three of the four product axes: **instant** (single-pass pipeline, no second-stage LLM), **private** (local-first, ephemeral audio, Keychain secrets, single-provider egress), and **everywhere** (global hotkey, press-time target capture across apps and monitors). The fourth axis — *gets better at how Kees speaks* — is explicitly v2.

## In scope (this blueprint)

Scribe v1 is a background, menu-bar-resident native macOS app that does one job: capture speech on a global hotkey and paste the transcribed text back into the field that was focused when the key was pressed. Single pass, local-first, no accounts. The Brief is the operative document; where the prior wiki blueprint contradicts it, the Brief wins. Two deltas from the prior wiki blueprint are now resolved in v1's favour: (1) the **vocabulary / custom-prompt field is in v1** (Sprint 3 Settings), and (2) the **default ships as Groq `whisper-large-v3-turbo`**.

## User surfaces

One user, one role: the owner (Kees), on his own Mac. Four surfaces:

- **Capture / dictation HUD (the pill).** Non-focus-stealing overlay that shows Scribe is listening (live waveform) and transcribing (animated linger), so the transcript lands in the press-time target field without breaking flow.
- **Menu-bar presence.** Discrete state icon (idle / listening / sending / done / error) so the owner knows app state at a glance, plus an "Open Settings" item. Minimal idle stub ships in Sprint 1; full state machine in Sprint 3.
- **Onboarding wizard (first-run).** Single guided sequence for the four OS-permission and key-setup steps, ending in a working test dictation. Sprints 1 (steps 1–4) and 2 (steps 5–6 + test recording).
- **Settings window.** One place to tune hotkey, capture mode, provider, model, API keys, vocabulary prompt, transcripts list with retry, and retention window.

A v2 correction / learning surface is named here only so the SQLite schema stays forward-compatible. Not designed in v1.

## Feature inventory

**Capture / dictation HUD**
- Global hotkey activation across any focused field, any app, any monitor.
- Hold-to-talk and tap-to-toggle capture modes.
- Capture pill with live waveform; non-activating NSPanel so the target field stays frontmost.
- "Transcribing…" animated linger state from key-release until paste lands — no silent gap.
- Paste into the press-time target field, multi-monitor correct, rendered on the active screen's frame (not the primary display's).
- Clipboard save → set transcript → synthetic ⌘V → restore original clipboard.
- Transient error / offline / paste-blocked banner — never a silent hang or garbage paste.
- Failed-transcript persistence so a failed dictation is recoverable from Settings.

**Menu-bar presence**
- Sprint 1: idle icon stub (so the user has a visible anchor that the app is running).
- Sprint 3: full state machine (idle / listening / sending / done / error) + "Open Settings".

**Onboarding wizard (first-run)**
- Step 1: Microphone permission with live granted/denied status.
- Step 2: Accessibility permission with live status, deep-link to the right System Settings pane, and copy that explicitly warns "Re-granting may be needed after every app update."
- Step 3: API key entry for at least one provider, shown with the verbatim egress-disclosure copy from `branding.md` plus a one-sentence disclosure of the selected provider's current audio-retention policy.
- Step 4: Hotkey recording — pre-filled non-conflicting default, conflict-checked at bind time.
- Step 5: Provider and model selection — default Groq `whisper-large-v3-turbo`.
- Closing test recording — speak a sample, see the transcript without pasting; pipeline integration gate that closes the wizard.

**Settings window**
- Hotkey remap.
- Capture mode toggle (hold-to-talk / tap-to-toggle).
- Provider selection (Groq / OpenAI) and model selection per provider.
- API key management per provider (Keychain-backed; enter, replace, remove).
- Vocabulary / custom-prompt field — a plain textarea (not a tag-input) passed as the STT biasing prompt to cut NZ-accent and personal-vocab mishears.
- Test transcription — speak a sample, display the transcript, no paste.
- Recent transcripts list with retry action for failed rows.
- Retention window (default 7 days, configurable).

## Sequencing rationale

- **Sprint 0 — throwaway de-risk spike, before any product code.** Two existential unknowns gate the entire product: (a) a Tauri window backed by `NSWindowStyleMaskNonActivatingPanel` genuinely never steals focus on the target Mac, and (b) `tauri-plugin-global-shortcut` (or a fallback `rdev` / `CGEventTap`) emits key-release events so hold-to-talk works. Neither can be answered by reading docs — they require Rust FFI on a real Mac. Sprint 0 also pins `tauri-nspanel` to a specific commit hash and inspects that commit, because the spike's vehicle is the most privileged native surface in the app.
- **Sprint 1 — scaffold, onboarding steps 1–4, idle tray stub.** Sprint 1 closes the cold-start cliff: Tauri 2 + Rust scaffold, `shared` constants, `secrets` (Keychain), `platform/macos` skeleton, capability allow-lists per window, and Onboarding Steps 1–4 (Mic, Accessibility, API key, hotkey). A ten-line idle-icon tray stub lands here so the user has a visible anchor through Sprint 2 (the Product voice surfaced this gap; the Technical voice accepted it). The full Universal `.app` signed/notarized build path is validated end-to-end in Sprint 1 to surface FFI breakage early. `cargo deny` config is added when `Cargo.toml` is created.
- **Sprint 2 — full dictation pipeline + onboarding steps 5–6 + capture pill.** End-to-end happy path: hotkey → capture → STT → delivery → SQLite persistence. **Failure states ship in Sprint 2 alongside the happy path, not as polish** — offline banner, provider-error banner, paste-blocked clean failure, failed-transcript persistence. Scribe is a daily-use tool; a silent hang on the first bad network destroys trust before the tool proves itself. The SQLite file is created with `0600` permissions in this sprint (the file does not exist before `storage` is built, so this is the right home). Sprint 2 also includes the multi-monitor pill placement AC: pill renders on the active screen's frame, not the primary display's.
- **Sprint 3 — Settings window + full tray + vocabulary prompt.** Settings unlocks the vocabulary prompt (Kees's accent-accuracy lever), provider/model switching, and transcript retry. The full `TrayState` machine and "Open Settings" item move from stub to real here. The vocabulary prompt UI is a plain textarea using existing `brand-book.md` input primitives — no new component spec.
- **Sprint 4 — hardening + packaging.** Signed + notarized Universal `.app` / `.dmg` cut from a clean `cargo audit` pass. Edge-case hardening on permission revocation, retention boundary, and clipboard-restore failure paths. No new product features.

## Out of scope

- v2 LLM cleanup pass (cheap Gemini second stage). Single-pass is a v1 invariant.
- v2 correction / learning layer and personal glossary. The fourth axis is deferred.
- Keystroke-typing fallback for paste. Clean failure is preferred — `rules-and-constraints.md` Must-not.
- Claude or the Claude Agent SDK inside the app. Explicit owner decision.
- Accounts, authentication, login, multi-user, roles. Permanently out.
- Cloud database, cloud sync, web version. Permanently out — local-only.
- Windows support in v1 (macOS-only). Kept behind a `Platform` trait so a future port is an add, not a rewrite.
- Spend guard / usage cap, auto-updater, App Store distribution, language auto-detection (hard-coded `en`).

## Risks (cross-cutting)

1. **NSPanel focus-steal (existential).** If the overlay steals focus, the dictation target changes and the product premise breaks. Mitigation: Sprint 0 spike with a single pass criterion — "TextEdit stays frontmost while overlay is visible." No product code starts before this is green.
2. **Key-up detection gap.** If `tauri-plugin-global-shortcut` does not emit key-release events, hold-to-talk is impossible. Mitigation: Sprint 0 spike resolves; fallback to `rdev` or `CGEventTap` via `objc2`/`core-graphics`. Trait surface stays narrow regardless.
3. **Privileged Tauri command caller spoofing.** Tauri IPC does not restrict which webview invokes a registered command. Mitigation: every privileged handler is dual-gated — capability JSON per window AND `window.label()` check in the handler body. Either alone is insufficient. Becomes an explicit AC bullet in every sprint that adds handlers.
4. **API key echo in STT error bodies.** Bearer tokens can be echoed in HTTP error responses; one log line leaks the key. Mitigation: `stt/sanitize.rs` strips `gsk_*` and `sk-*` patterns from any error body before it leaves the `stt` module; unit tests gate every error path.
5. **Universal binary FFI breakage.** `rusqlite`, `cpal`, `objc2` framework bindings, and `enigo` must compile cleanly for `aarch64-apple-darwin` and `x86_64-apple-darwin`. Mitigation: validate `cargo tauri build --target universal-apple-darwin` (signed + notarized) in Sprint 1 — not Sprint 4.
6. **Accessibility trust revocation on every rebuild.** macOS hashes the binary digest and silently revokes Accessibility on every dev rebuild, breaking synthetic ⌘V. Mitigation: `AXIsProcessTrustedWithOptions` startup check surfaces a modal before anything else; onboarding Step 2 copy warns explicitly.
7. **Onboarding cliff.** Two OS permissions + an API key before any value lands. Mitigation: live permission status, working deep-link to System Settings, verbatim egress copy, closing test recording as the wizard's completion gate.
8. **Provider data retention disclosure.** `security-framework.md` §2 requires documenting each provider's audio retention. None of the three voices surfaced this independently; the Wiki voice added it on critique. Mitigation: Sprint 1 onboarding Step 3 documents Groq's and OpenAI's current policies in-product alongside the egress copy.

## Open questions

- **None blocking.** The vocabulary-prompt contradiction (Brief in-v1 vs. prior wiki blueprint deferred) is resolved in favour of the Brief; the synthesis pass updates the wiki copy of `blueprint.md` and closes the open-question entry in `rules-and-constraints.md`.
- **Non-blocking detail confirmed during synthesis:** vocabulary prompt UI is a plain textarea (not a tag-input). A textarea reuses existing brand-book input primitives and avoids inventing a new data model. Reversible in Sprint 3 if Kees prefers tags after using it.
- **Non-blocking edge-case AC:** "No field focused at press" must appear as an explicit Sprint 2 AC bullet — paste no-ops with quiet indication, transcript still saved.
- **None of the three voices explored the live Mac.** Sprint 0's pass/fail is the only real answer to the NSPanel and key-up questions; everything else is desk reasoning.