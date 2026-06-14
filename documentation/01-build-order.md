# Build Order — Scribe, blueprint v1

Total sprints: 5

## Sequence

1. **Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection** — Prove the two existential macOS-FFI unknowns that gate the entire product before any product code is written. Either (a) a Tauri window backed by NSWindowStyleMaskNonActivatingPanel genuinely does not steal focus, and (b) tauri-plugin-global-shortcut (or a fallback) emits key-release events for hold-to-talk — both go/no-go. Throwaway spike: no shippable code, only architectural confidence. Pin and vet tauri-nspanel as part of the spike since it is the implementation vehicle for the most privileged native surface.
   Depends on: none
   Surface: infra
2. **Sprint 1 — Scaffold, secrets, onboarding steps 1–4, tray idle stub, Universal build path** — Stand up the Tauri 2 + Rust + React project skeleton, ship the first four onboarding steps (Microphone, Accessibility, API key, hotkey), wire the macOS Keychain for secrets, and validate the Universal signed + notarized build path end-to-end. A minimal idle-icon tray stub lands so the user has a visible anchor through Sprint 2. cargo deny is configured. No dictation pipeline yet — Sprint 2's job.
   Depends on: 1
   Surface: desktop-app
3. **Sprint 2 — Full dictation pipeline + capture pill + onboarding steps 5–6 + storage with 0600** — Wire the end-to-end dictation pipeline so Scribe becomes a daily-use tool: hotkey → capture → STT → delivery → SQLite. Ship both capture modes, the non-activating capture pill with live waveform and "Transcribing…" linger, multi-monitor-correct paste into the press-time target field, clipboard save/restore, and the full failure-handling surface (offline, provider error, paste-blocked, failed-transcript persistence). Storage created with 0600 permissions. Onboarding Steps 5 (provider/model picker) and 6 (closing test recording) complete the wizard.
   Depends on: 1, 2
   Surface: desktop-app
4. **Sprint 3 — Settings window, full tray state machine, vocabulary prompt** — Ship the Settings window (the single place to tune every configurable aspect of Scribe), upgrade the tray stub to the full state machine, and wire the vocabulary prompt — Kees's accent-accuracy lever — so the NZ accent and his personal vocabulary stop tripping up transcription.
   Depends on: 1, 2, 3
   Surface: desktop-app
5. **Sprint 4 — Hardening, edge-case audit, signed + notarized release build** — Convert Sprint 2 + 3's working tool into a stable installable build. Audit every edge case from rules-and-constraints.md and user-flow-testing.md against the running app, run the security-tooling pass (cargo audit), and cut the final signed + notarized Universal .app / .dmg. No new product features.
   Depends on: 1, 2, 3, 4
   Surface: infra

## Dependency graph

- (root) → 1. Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection
- 1. Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection → 2. Sprint 1 — Scaffold, secrets, onboarding steps 1–4, tray idle stub, Universal build path
- 1. Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection + 2. Sprint 1 — Scaffold, secrets, onboarding steps 1–4, tray idle stub, Universal build path → 3. Sprint 2 — Full dictation pipeline + capture pill + onboarding steps 5–6 + storage with 0600
- 1. Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection + 2. Sprint 1 — Scaffold, secrets, onboarding steps 1–4, tray idle stub, Universal build path + 3. Sprint 2 — Full dictation pipeline + capture pill + onboarding steps 5–6 + storage with 0600 → 4. Sprint 3 — Settings window, full tray state machine, vocabulary prompt
- 1. Sprint 0 — De-risk spike: non-activating NSPanel + key-up detection + 2. Sprint 1 — Scaffold, secrets, onboarding steps 1–4, tray idle stub, Universal build path + 3. Sprint 2 — Full dictation pipeline + capture pill + onboarding steps 5–6 + storage with 0600 + 4. Sprint 3 — Settings window, full tray state machine, vocabulary prompt → 5. Sprint 4 — Hardening, edge-case audit, signed + notarized release build

## Sequencing rationale

- **Sprint 0 — throwaway de-risk spike, before any product code.** Two existential unknowns gate the entire product: (a) a Tauri window backed by `NSWindowStyleMaskNonActivatingPanel` genuinely never steals focus on the target Mac, and (b) `tauri-plugin-global-shortcut` (or a fallback `rdev` / `CGEventTap`) emits key-release events so hold-to-talk works. Neither can be answered by reading docs — they require Rust FFI on a real Mac. Sprint 0 also pins `tauri-nspanel` to a specific commit hash and inspects that commit, because the spike's vehicle is the most privileged native surface in the app.
- **Sprint 1 — scaffold, onboarding steps 1–4, idle tray stub.** Sprint 1 closes the cold-start cliff: Tauri 2 + Rust scaffold, `shared` constants, `secrets` (Keychain), `platform/macos` skeleton, capability allow-lists per window, and Onboarding Steps 1–4 (Mic, Accessibility, API key, hotkey). A ten-line idle-icon tray stub lands here so the user has a visible anchor through Sprint 2 (the Product voice surfaced this gap; the Technical voice accepted it). The full Universal `.app` signed/notarized build path is validated end-to-end in Sprint 1 to surface FFI breakage early. `cargo deny` config is added when `Cargo.toml` is created.
- **Sprint 2 — full dictation pipeline + onboarding steps 5–6 + capture pill.** End-to-end happy path: hotkey → capture → STT → delivery → SQLite persistence. **Failure states ship in Sprint 2 alongside the happy path, not as polish** — offline banner, provider-error banner, paste-blocked clean failure, failed-transcript persistence. Scribe is a daily-use tool; a silent hang on the first bad network destroys trust before the tool proves itself. The SQLite file is created with `0600` permissions in this sprint (the file does not exist before `storage` is built, so this is the right home). Sprint 2 also includes the multi-monitor pill placement AC: pill renders on the active screen's frame, not the primary display's.
- **Sprint 3 — Settings window + full tray + vocabulary prompt.** Settings unlocks the vocabulary prompt (Kees's accent-accuracy lever), provider/model switching, and transcript retry. The full `TrayState` machine and "Open Settings" item move from stub to real here. The vocabulary prompt UI is a plain textarea using existing `brand-book.md` input primitives — no new component spec.
- **Sprint 4 — hardening + packaging.** Signed + notarized Universal `.app` / `.dmg` cut from a clean `cargo audit` pass. Edge-case hardening on permission revocation, retention boundary, and clipboard-restore failure paths. No new product features.
