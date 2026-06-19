# Sprint 1 Security Audit — Scribe

**Date:** 2026-06-19
**Auditor:** Orion (SCRIBE-7)
**Scope:** Full Sprint 1 surface — modules `shared`, `secrets`, `platform/macos`, `onboarding`, `tray`; capability JSONs; `tauri.conf.json`; `Cargo.toml` / `Cargo.lock`; egress-copy snapshot test.
**Verdict:** PASS — 1 inline fix applied, 0 escalated, 4 Sprint-2-deferred items listed.

---

## Production-vs-throwaway determination

**Production-bound.** This is the first real product code for Scribe, not a throwaway spike. Full audit weight applied per the council's keep-it-full decision (Sprint 34). The findings below are treated as blocking for the Sprint 1 surface they cover.

---

## Security-framework MUSTs (§1–12)

| § | MUST | Finding | Evidence |
|---|---|---|---|
| 1 | Threat model — audio path | **DEFERRED to Sprint 2** | Audio capture/transcription not yet built. Will be audited in Sprint 2 alongside actual audio pipeline. |
| 2 | Data handling — SQLite at 0o600 | **DEFERRED to Sprint 2** | SQLite store not yet created. Sprint 2 must enforce mode 0o600 on the DB file. |
| 3 | Authentication — none by design | **pass** | No auth code, no accounts. `security-framework.md:36` confirms single-user, single-machine. |
| 4 | Tenant isolation — single-tenant by design | **pass** | No multi-tenant code. Data lives under `~/Library/Application Support/Scribe/` with mode 0o700. |
| 5 | Secrets management — paste/clipboard | **DEFERRED to Sprint 2** | Clipboard paste/restore not yet implemented. Sprint 2 must verify clipboard is restored after every paste. |
| 6 | Permissions & platform trust | **pass** | Onboarding wizard gates Mic → AX → API key → hotkey. Capability allow-lists are minimal per-window. `tauri.conf.json:13` enables `macOSPrivateApi` (required for AX + IOKit HID). |
| 7 | Egress at runtime | **DEFERRED to Sprint 2** | No actual audio/text egress yet (providers not called). Sprint 2 must verify audio sent only to selected provider. |
| 8 | Dependency / supply-chain | **pass** | Dependencies pinned in `Cargo.toml` / committed `Cargo.lock`. `cargo deny check` green (advisories ok, bans ok, licenses ok, sources ok). `deny.toml:28` sets `unknown-git = "warn"` (deliberate — `tauri-nspanel` is git-sourced at pinned rev). |
| 9 | Local data perms — 0o700 dir, 0o600 file | **pass** | `secrets/mod.rs:77` sets mode 0o700 on `app_data_dir()`. `onboarding/persistence.rs:51` sets mode 0o600 on `onboarding.json`. Both enforced by unit tests at `secrets/mod.rs:196` and `onboarding/persistence.rs:124`. |
| 10 | Graceful failure leaks nothing | **pass** | `SecretsError` Display says "keychain error" / "io error" — never the key value. Test at `secrets/mod.rs:179` confirms error Display never contains a test key. `OnboardingError` Display shows "io error: {e}" / "serde error: {e}" — no secrets. |
| 11 | No telemetry by default | **pass** | `grep -rE 'sentry|honeycomb|posthog|analytics|telemetry' Cargo.toml Cargo.lock` returns nothing. Only `tracing` + `tracing-appender` + `tracing-subscriber` for local log file at `~/Library/Logs/Scribe/scribe.log`. |
| 12 | Privileged command scoping | **pass** | See per-command audit below. All privileged commands have `window.label()` guards. Unprivileged read-only commands are capability-scoped only. |

---

## Four named greps

### (a) KEYCHAIN_SERVICE — `"Scribe"` locations

Command: `grep -rE '"Scribe"' src-tauri/src/ src/`

```
src-tauri/src/shared/mod.rs:pub const APP_NAME: &str = "Scribe";
src-tauri/src/shared/mod.rs:pub const KEYCHAIN_SERVICE: &str = "Scribe";
src-tauri/src/shared/mod.rs:assert_eq!(super::KEYCHAIN_SERVICE, "Scribe");
src/shared/index.ts:export const APP_NAME = "Scribe" as const;
src/shared/index.ts:export const KEYCHAIN_SERVICE = "Scribe" as const;
src-tauri/src/secrets/mod.rs:.join("Scribe");
src-tauri/src/secrets/mod.rs:assert!(dir.ends_with("Scribe"));
```

**Verdict: pass.** KEYCHAIN_SERVICE is declared only in the shared module (Rust `shared/mod.rs:7` + TS `shared/index.ts:2`) and its test. The `"Scribe"` usages in `secrets/mod.rs` are for the data directory name (not the Keychain service), which is correct.

---

### (b) FFI imports outside platform/macos/

Command: `grep -rE 'use objc2|use core_graphics|use objc2_io_kit|use objc2_app_kit' src-tauri/src/`

All matches are exclusively under `src-tauri/src/platform/macos/`:

```
src-tauri/src/platform/macos/mod.rs:            use objc2::msg_send;
src-tauri/src/platform/macos/mod.rs:            use objc2::rc::Retained;
src-tauri/src/platform/macos/mod.rs:            use objc2_foundation::{NSDictionary, NSNumber, NSString};
src-tauri/src/platform/macos/permissions.rs:use objc2::msg_send;
src-tauri/src/platform/macos/permissions.rs:use objc2::rc::Retained;
src-tauri/src/platform/macos/permissions.rs:use objc2_foundation::NSString;
src-tauri/src/platform/macos/permissions.rs:    use objc2::runtime::Bool;
```

**Verdict: pass.** All `objc2`, `objc2_foundation`, and related FFI imports are confined to the `platform/macos/` module. No `core_graphics` or `objc2_app_kit` imports exist yet (the `macOSPrivateApi` flag in `tauri.conf.json:13` enables them when needed).

---

### (c) tauri-nspanel branch/tag

Command: `grep -rE 'tauri-nspanel.*(branch|tag)' src-tauri/Cargo.toml src-tauri/Cargo.lock`

**No matches.** `Cargo.toml:17` pins `tauri-nspanel` by `rev = "a3122e894383aa068ec5365a42994e3ac94ba1b6"` — a specific commit SHA. No `branch` or `tag` is used.

**Verdict: pass.** Pinned by rev, which is the most secure approach (immutable git reference). Vetted during Sprint 0 Gate C.

---

### (d) Telemetry crates

Command: `grep -rE 'sentry|honeycomb|posthog|analytics|telemetry' src-tauri/Cargo.toml src-tauri/Cargo.lock`

**No matches.** The only observability crates are `tracing`, `tracing-appender`, and `tracing-subscriber` — all writing to a local log file. No network telemetry.

**Verdict: pass.**

---

## Per-command audit

Every `#[tauri::command]` in `src-tauri/src/lib.rs` cross-checked against capability JSONs and `window.label()` guards.

| # | Command | Window check | Source capability | Correct scope? |
|---|---|---|---|---|
| 1 | `set_api_key_cmd` | `ALLOWED_SECRETS_WINDOWS.contains(&window.label())` | onboarding.json | pass — privileged, dual-gated |
| 2 | `delete_api_key_cmd` | `ALLOWED_SECRETS_WINDOWS.contains(&window.label())` | onboarding.json | pass — privileged, dual-gated |
| 3 | `has_api_key_cmd` | `ALLOWED_SECRETS_WINDOWS.contains(&window.label())` | onboarding.json | pass — privileged, dual-gated |
| 4 | `request_microphone_permission_cmd` | `ALLOWED_SECRETS_WINDOWS.contains(&window.label())` | onboarding.json | pass — privileged, dual-gated |
| 5 | `is_onboarding_complete_cmd` | none (non-privileged read) | main.json | pass — read-only, capability-scoped |
| 6 | `mark_onboarding_complete_cmd` | `window.label() != "onboarding" → Err` | onboarding.json | pass — privileged, dual-gated |
| 7 | `open_onboarding_cmd` | none (uses AppHandle, opens window) | main.json | pass — non-privileged UI action |
| 8 | `close_onboarding_cmd` | `window.label() != "onboarding" → return` | onboarding.json | pass — non-privileged, gated |
| 9 | `platform_microphone_status_cmd` | none (non-privileged read) | onboarding.json | pass — read-only, capability-scoped |
| 10 | `platform_accessibility_status_cmd` | none (non-privileged read) | onboarding.json | pass — read-only, capability-scoped |
| 11 | `platform_open_microphone_pane_cmd` | none (non-privileged, opens system prefs) | onboarding.json | pass — capability-scoped |
| 12 | `platform_open_accessibility_pane_cmd` | none (non-privileged, opens system prefs) | onboarding.json | pass — capability-scoped |
| 13 | `platform_hotkey_conflicts_cmd` | none (non-privileged query) | onboarding.json | pass — capability-scoped |
| 14 | `platform_register_hotkey_cmd` | none (uses AppHandle, not window) | onboarding.json | pass — gated by capability; note: if Tauri later adds window-scoped hotkey registration, add a window.label() guard. |

**Capability JSON permission summary:**

| Capability | Windows | Privileged commands | Read-only commands | Empty? |
|---|---|---|---|---|
| `main.json` | `["main"]` | none | `is-onboarding-complete-cmd`, `open-onboarding-cmd` | no |
| `onboarding.json` | `["onboarding"]` | set/delete/has-api-key-cmd, request-microphone-permission-cmd, mark-onboarding-complete-cmd | 7 platform/onboarding commands | no |
| `overlay.json` | `["overlay"]` | none | none | yes (correct — no commands needed) |
| `settings.json` | `["settings"]` | none | none | yes (correct — not populated yet) |

**Permission manifests** are in `src-tauri/permissions/` and map `snake_case` Rust command names to Tauri's hyphenated capability identifiers:
- `secrets.toml` — `set_api_key_cmd`, `delete_api_key_cmd`, `has_api_key_cmd`
- `platform.toml` — `request_microphone_permission_cmd` + 6 platform commands
- `onboarding.toml` — `is_onboarding_complete_cmd`, `mark_onboarding_complete_cmd`, `open_onboarding_cmd`, `close_onboarding_cmd`

**Verdict: pass.** All 14 commands are correctly capability-scoped. The 5 privileged commands (1–4, 6) have explicit `window.label()` guards. The 9 unprivileged commands are appropriately gated by capability scoping alone.

---

## Module-grep audits

### (e) secrets/ — key-leaking log calls

Command: `grep -rn 'tracing::\|log::' src-tauri/src/secrets/`

**No matches.** The secrets module contains no `tracing::*!` or `log::*!` calls whatsoever.

**Verdict: pass.**

### (f) platform/macos/ — payload-leaking log calls

Command: `grep -rn 'tracing::\|log::' src-tauri/src/platform/macos/`

```
src-tauri/src/platform/macos/hotkey_fn.rs:96:  tracing::info!(binding = %HotkeyBinding::Fn, event = ?HotkeyEvent::Press);
src-tauri/src/platform/macos/hotkey_fn.rs:99:  tracing::info!(binding = %HotkeyBinding::Fn, event = ?HotkeyEvent::Release);
src-tauri/src/platform/macos/hotkey_fn.rs:117: tracing::error!("IOHIDManagerCreate returned null");
src-tauri/src/platform/macos/hotkey_fn.rs:140: tracing::error!(result, "IOHIDManagerOpen failed");
src-tauri/src/platform/macos/hotkey_fn.rs:149: tracing::info!("fn hotkey tap running");
src-tauri/src/platform/macos/mod.rs:83:        tracing::error!(?e, "failed to start fn hotkey tap");
src-tauri/src/platform/macos/hotkey_plugin.rs:24: tracing::info!(binding = %HotkeyBinding::ChordCtrlOptSpace, event = ?HotkeyEvent::Press);
src-tauri/src/platform/macos/hotkey_plugin.rs:31: tracing::info!(binding = %HotkeyBinding::ChordCtrlOptSpace, event = ?HotkeyEvent::Release);
```

All 8 calls log only hotkey binding types (`Fn`, `ChordCtrlOptSpace`) and event types (`Press`, `Release`) or internal error states. **None interpolate audio data, event payload content, or user data.**

**Verdict: pass.**

---

## Cross-task tests-exist checks

### Scaffold test deletion (SCRIBE-1 placeholder)

The SCRIBE-1 placeholder test `scaffold_compiles()` at `lib.rs:188` was an empty no-op Rust test. The AC required the vitest placeholder to be deleted by SCRIBE-2 (the vitest placeholder was indeed deleted). The Rust `scaffold_compiles()` test remained as vestigial tech debt.

**Finding: inline-fixed.** Removed `scaffold_compiles()` from `lib.rs:188`. Cargo test re-run confirms 47/47 pass (was 48 with the empty scaffold). `test:fast` vitest component tests fail with missing `@testing-library/react` (pre-existing worktree dependency issue — unrelated to this change).

### ALLOWED_SECRETS_WINDOWS single declaration

**Verdict: pass.** Declared exactly once at `src-tauri/src/shared/mod.rs:8`: `pub const ALLOWED_SECRETS_WINDOWS: &[&str] = &["onboarding"];`. Imported and used in `lib.rs` only. No other declarations exist.

### app-data dir mode 0o700 test

**Verdict: pass.** Test `app_data_dir_creates_and_has_mode_700` at `secrets/mod.rs:196` exists and passes. Sets `HOME` to temp dir, calls `app_data_dir()`, asserts `mode & 0o777 == 0o700`.

### onboarding.json mode 0o600 test

**Verdict: pass.** Test `file_mode_is_0600` at `onboarding/persistence.rs:124` exists and passes. Writes `onboarding.json` via `write_completed()`, asserts `mode & 0o777 == 0o600`.

---

## Additional checks

### tauri.conf.json

- `bundle.macOS.signingIdentity: "-"` — ad-hoc signing only ✓
- No `updater` block ✓
- `macOSPrivateApi: true` required for AX + IOKit HID ✓
- Four window definitions: `main`, `onboarding`, `overlay`, `settings` ✓

### Cargo.toml dependency surface

- `objc2` (0.6), `objc2-app-kit` (0.3), `objc2-foundation` (0.3), `objc2-io-kit` (0.3) — confined to `platform/macos/` ✓
- `keyring` (4.0), `keyring-core` (1.0) — confined to `secrets/` ✓
- `tauri-nspanel` — git-sourced, pinned by rev ✓
- `tracing`, `tracing-appender`, `tracing-subscriber` — local file logging only ✓
- No `claude-sdk`, `anthropic`, `openai`, `groq` crates — not yet needed ✓

### Cargo.lock

Committed ✓. Contains no branch/tag references for `tauri-nspanel` ✓.

### egress-copy snapshot test

`src/shared/__tests__/egress-copy.test.ts` — inline snapshots match the verbatim egress disclosure copy from `shared/index.ts`. Both Groq and OpenAI variants tested.

---

## Sprint-2-deferred items

These four security-framework MUSTs are explicitly deferred to Sprint 2. The Sprint 2 breakdown reviewer must carry them forward.

| § | Item | One-line citation |
|---|---|---|
| 1 | Audio path (no raw audio on disk) | Audio capture not built yet. Sprint 2 must verify no audio file/buffer persisted after dictation. |
| 2 | SQLite at 0o600 | SQLite store not created yet. Sprint 2 must enforce `mode 0o600` on the transcript database file. |
| 5 | Paste / clipboard restore | Clipboard paste/restore not implemented. Sprint 2 must verify previous clipboard is saved and restored after every paste. |
| 7 | Egress scoped to selected provider | No actual provider API calls yet. Sprint 2 must verify audio/text sent only to `selected provider` endpoint. |

---

## Summary

| Category | Count |
|---|---|
| Pass | 28 |
| Inline-fixed | 1 (scaffold test removal) |
| Escalated | 0 |
| Deferred to Sprint 2 | 4 (§1, §2, §5, §7) |

**Inline fix applied:** Removed empty `scaffold_compiles()` Rust test at `lib.rs:188`. Cargo deny green, all 47 Rust tests pass.

**No structural gaps found.** The four privileged commands are properly dual-gated (capability + `window.label()`) as required by `security-framework.md` §12. The unprivileged commands are correctly capability-scoped per-window. No secrets leak into logs, no telemetry crates exist, and the Keychain service name is declared only in the shared module.
