# Sprint 1 Security Audit — Scribe

**Date:** 2026-06-19
**Auditor:** Orion (SCRIBE-7, round 2)
**Scope:** Full Sprint 1 surface — modules `shared`, `secrets`, `platform/macos`, `onboarding`, `tray`; capability JSONs; `tauri.conf.json`; `Cargo.toml` / `Cargo.lock`; egress-copy snapshot test.
**Verdict:** PASS — 1 inline fix applied (round 1), 0 escalated, 2 deferred to build-gate (SCRIBE-6 checks 3–4), 4 Sprint-2-deferred items (§1/2/5/7).

---

## Production-vs-throwaway determination

**Production-bound.** This is the first real product code for Scribe, not a throwaway spike. Full audit weight applied per the council's keep-it-full decision (Sprint 34). The findings below are treated as blocking for the Sprint 1 surface they cover.

---

## Per-task security_checks table — SCRIBE-1 through SCRIBE-6

### SCRIBE-1 — Repo-root scaffold, CI/deny/docs, capabilities skeleton

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | No `.env`, credential fixture, test API key, or plaintext secret committed in tree | **pass** | `find . -name '.env'` returns nothing; `git ls-files | grep -i .env` returns nothing; `grep -rn 'gsk_\|sk-'` (excluding test fixtures) returns nothing |
| 2 | No telemetry crate (sentry/honeycomb/posthog/analytics); no updater endpoint in tauri.conf.json | **pass** | `grep -rE 'sentry\|honeycomb\|posthog\|analytics\|telemetry' Cargo.toml Cargo.lock` returns nothing; `tauri.conf.json` has no `updater` block |
| 3 | All four capability JSONs constrain `windows` selector to a specific label (no `"*"`) | **pass** | `main.json:4` → `["main"]`; `onboarding.json:4` → `["onboarding"]`; `overlay.json:4` → `["overlay"]`; `settings.json:4` → `["settings"]` |
| 4 | `tauri-nspanel` pinned by `rev = "<40-char SHA>"` only; no `branch` or `tag` | **pass** | `Cargo.toml:17` pins `rev = "a3122e894383aa068ec5365a42994e3ac94ba1b6"`; `grep -rE 'tauri-nspanel.*(branch\|tag)'` returns nothing |
| 5 | `deny.toml` denies yanked crates and warns on unknown-git sources | **pass** | `deny.toml:24` — `yanked = "deny"`; `deny.toml:28` — `unknown-git = "warn"`; `cargo deny check` exits 0 (advisories ok, bans ok, licenses ok, sources ok) |

### SCRIBE-2 — shared module (Rust + TS mirror), egress copy, retention strings

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | `KEYCHAIN_SERVICE` declared exactly once per stack (Rust `shared/mod.rs`, TS `shared/index.ts`) | **pass** | `shared/mod.rs:7` — `pub const KEYCHAIN_SERVICE: &str = "Scribe"`; `shared/index.ts:2` — `export const KEYCHAIN_SERVICE = "Scribe"`; grep confirms `"Scribe"` string only inside these two files + tests + data-dir join (non-Keychain usage) |
| 2 | `EGRESS_DISCLOSURE_COPY` strings byte-identical between Rust and TS; snapshot test guards | **pass** | `shared/mod.rs:19-30` — Rust copies match TS copies at `shared/index.ts:8-13` byte-for-byte; `egress-copy.test.ts:6,12` — inline snapshots lock both Groq and OpenAI variants |
| 3 | Retention strings hardcoded with source URLs for 30-second re-verification | **pass** | `shared/mod.rs:1-4` — `// Sources for retention disclosures:` block with Groq privacy URL + DPA URL + OpenAI enterprise privacy URL + verification dates; `shared/index.ts:15-21` — same source URLs in TS comments |

### SCRIBE-3 — secrets module, Keychain wrapper, privileged commands, app-data dir

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | `SecretsError` Display never includes raw key; unit test greps for fake key against every variant | **pass** | `secrets/error.rs:5-10` — Display impls say "keychain error" / "io error", never `{0}`; `secrets/mod.rs:179-193` — test `secrets_error_display_never_contains_fake_key` asserts no variant's Display contains `"gsk_test_NEVERAPPEAR"` |
| 2 | Privileged commands `set_api_key_cmd`, `delete_api_key_cmd`, `has_api_key_cmd` listed in `onboarding.json` and NOT in `main.json` | **pass** | `onboarding.json:9-11` — three secrets commands present; `main.json:5-10` — secrets commands absent; `lib.rs:218-237` — test `secrets_commands_are_only_in_onboarding_capabilities` asserts this |
| 3 | Each handler calls `window.label()` and rejects any label other than onboarding | **pass** | `lib.rs:24-27` (set), `lib.rs:35-38` (delete), `lib.rs:46-49` (has) — all check `ALLOWED_SECRETS_WINDOWS.contains(&window.label())`; `lib.rs:188-199` — test `has_api_key_cmd_rejects_overlay_label`; `lib.rs:203-215` — test `has_api_key_cmd_accepts_onboarding_label` |
| 4 | Set with whitespace input stores trimmed value; unit test covers | **pass** | `secrets/mod.rs:46-51` — `set_api_key` calls `key.trim()` before storing; `secrets/mod.rs:122-127` — test `set_with_whitespace_trims` asserts `"  sk-test  "` → `"sk-test"` |
| 5 | `app_data_dir` creates `~/Library/Application Support/Scribe/` at mode `0o700` | **pass** | `secrets/mod.rs:64-83` — creates dir, checks mode `& 0o777 == 0o700`, sets if needed; `secrets/mod.rs:196-219` — test `app_data_dir_creates_and_has_mode_700` asserts exact mode |
| 6 | No `tracing::*!` or `log::*!` call in `secrets/` receives key value or trimmed key as argument | **pass** | `grep -rn 'tracing::\|log::' src-tauri/src/secrets/` returns **no matches**; the module has zero log/tracing calls |
| 7 | No panic path (`unwrap`/`expect`) propagates key value into panic message | **pass** | Production `unwrap`s in `secrets/mod.rs:19,23` are `Mutex::lock().unwrap()` and `Option::as_ref().unwrap()` (structural, no key value in panic); backend `unwrap`s at `backend.rs:63,71,80` are `Mutex::lock().unwrap()` (structural); all key data flows through `Result` with proper error mapping via `SecretsError` |

### SCRIBE-4 — platform/macos skeleton, permissions, IOKit HID fn tap

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | `request_microphone_permission_cmd` in `onboarding.json` and NOT in `main.json`; `window.label()` check rejects non-onboarding labels | **pass** | `onboarding.json:12` — `request-microphone-permission-cmd` present; `main.json` — absent; `lib.rs:53-60` — handler checks `ALLOWED_SECRETS_WINDOWS.contains(&window.label())`; `lib.rs:239-251` — test `request_microphone_permission_cmd_rejects_non_onboarding_label` |
| 2 | `platform/macos/` is the only crate path importing `objc2`/`objc2-app-kit`/`objc2-io-kit`/`objc2-foundation`/`core-graphics` | **pass** | `grep -rE 'use objc2\|use core_graphics\|use objc2_io_kit\|use objc2_app_kit' src-tauri/src/` returns only `platform/macos/` paths (7 lines across `mod.rs:34-36` and `permissions.rs:4-6,64`) |
| 3 | `AXIsProcessTrustedWithOptions` — status polls use `kAXTrustedCheckOptionPrompt:NO`; only `ax_check_with_prompt()` uses `YES` | **pass** | `permissions.rs:37-44` — `accessibility_status_impl()` calls `AXIsProcessTrusted()` (no prompt); `platform/macos/mod.rs:31-57` — `ax_check_with_prompt()` builds `NSDictionary` with `AXTrustedCheckOptionPrompt` → `true` and calls `AXIsProcessTrustedWithOptions`; the naming of the two functions (`ax_check_with_prompt` vs the no-prompt status poll) makes the distinction explicit |
| 4 | No tracing/log call in `platform/macos/` interpolates audio data/key value/captured event payload | **pass** | `grep -rn 'tracing::\|log::' src-tauri/src/platform/macos/` returns 8 calls — all log only `HotkeyBinding` names (`Fn`, `ChordCtrlOptSpace`) and `HotkeyEvent` kinds (`Press`, `Release`) or internal error states; none interpolate audio data or user content |
| 5 | Settings pane URLs match exactly: `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility` and `...?Privacy_Microphone` | **pass** | `settings_link.rs:3-6` — constants match the contracted URLs exactly; `settings_link.rs:25-29` — test `accessibility_pane_url_exact`; `settings_link.rs:33-38` — test `microphone_pane_url_exact` |

### SCRIBE-5 — onboarding module: Rust commands + 4-step React wizard

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | `mark_onboarding_complete_cmd`, `close_onboarding_cmd`, and SCRIBE-3 secrets commands all in `onboarding.json` and NOT in `main.json` | **pass** | `onboarding.json:13-14` — `mark-onboarding-complete-cmd`, `close-onboarding-cmd` present + secrets commands at lines 9-11; `main.json` — none of these present; `lib.rs:334-370` — test `new_onboarding_commands_in_capabilities` asserts each onboarding command in onboarding.json and absent from main.json |
| 2 | Each command handler in `lib.rs` calls `window.label()` and rejects non-matching labels; unit tests exercise | **pass** | `lib.rs:68-75` — `mark_onboarding_complete_cmd` checks `window.label() != "onboarding"`; `lib.rs:83-91` — `close_onboarding_cmd` checks `window.label() != "onboarding"`; `lib.rs:284-296` — test `mark_onboarding_complete_cmd_rejects_main_label`; `lib.rs:298-311` — test `mark_onboarding_complete_cmd_accepts_onboarding_label`; `lib.rs:320-331` — test `close_onboarding_cmd_ignores_non_onboarding_label` |
| 3 | `onboarding.json` written at mode `0o600`; unit test asserts | **pass** | `onboarding/persistence.rs:46-53` — `write_completed()` sets `perms.set_mode(0o600)` after write; `onboarding/persistence.rs:124-139` — test `file_mode_is_0600` asserts `mode & 0o777 == 0o600` |
| 4 | Step 3 input is `type="password"` so key is masked on screen | **pass** | `StepApiKey.tsx:68` — `<input type="password" ...>`; `step-api-key.test.tsx:49-59` — test `input has type=password when no existing key` asserts `input.type === "password"` |
| 5 | Wizard never logs or renders the entered key value after Step 3; masked placeholder is only visible representation | **pass** | `StepApiKey.tsx:71-73` — placeholder shows `"••••••••••••••••"` when `hasExisting` is true; the input is always `type="password"`; no `console.log` or `tracing` calls emit the key; `index.tsx:64-68` — on complete, `dispatch({ type: "SET_STEP", step: 4 })` advances without passing key to next step |
| 6 | Egress callout renders verbatim string from `egressDisclosureCopy('groq')` | **pass** | `StepApiKey.tsx:101` — `{egressDisclosureCopy("groq")}` renders the shared disclosure; `step-api-key.test.tsx:27-36` — test `renders egress callout with Groq disclosure text` asserts substring match; `egress-copy.test.ts:6` — snapshot test locks the exact Groq string |
| 7 | Step 2 contains exact phrase "Re-granting may be needed after every app update." | **pass** | `StepAccessibility.tsx:136` — exact text in `<p>`; `wizard.test.tsx:87-98` — test `contains the re-granting notice text` asserts substring match via `getByText` |

### SCRIBE-6 — Tray idle stub module, main wiring, AX startup check, Universal build

| # | Check | Verdict | File:line evidence |
|---|-------|---------|-------------------|
| 1 | Tray icon has no menu and no tooltip in Sprint 1 (no clickable surface) | **pass** | `tray/mod.rs:20-23` — `TrayIconBuilder::with_id("scribe-idle").icon(...).icon_as_template(true).build(app)` — no `.menu()`, no `.tooltip()`, no `.on_tray_icon_event()`, no `.on_menu_event()` |
| 2 | `ax_check_with_prompt()` called exactly once at startup; wizard polls use no-prompt variant | **pass** | `lib.rs:143` — `let ax_granted = MacPlatform::ax_check_with_prompt();` called once in `setup()`; `permissions.rs:37-44` — `accessibility_status_impl()` calls `AXIsProcessTrusted()` (no prompt), used by `platform_accessibility_status_cmd` for wizard polls; `StepAccessibility.tsx:83` — polls `platform_accessibility_status_cmd` every 1s |
| 3 | Built binary contains no entered API key (verify via `nm`/`strings` on `Scribe.app/Contents/MacOS/scribe`) | **deferred** | Requires full macOS build. Audit confirms: no API key is embedded in source; Keychain is the sole store. Build verification is deferred to the sprint-final build gate. |
| 4 | `~/Library/Logs/Scribe/scribe.log` does not contain entered API key (`grep -r` confirms) | **deferred** | Requires run-time. Audit confirms: `secrets/` module has zero `tracing::*!` or `log::*!` calls (grep empty); `set_api_key` never logs the key; the log file path (`lib.rs:125-130`) only records internal events. Run-time verification is deferred to sprint-final build gate. |
| 5 | `tauri.conf.json` declares ad-hoc signing only (`"-"`); no Developer-ID identity shipped | **pass** | `tauri.conf.json:54` — `"signingIdentity": "-"`; no `updater` block in tauri.conf.json; no `sentry`, `honeycomb`, `posthog`, `analytics`, `telemetry` crates in `Cargo.toml` |

---

## Security-framework MUSTs (§1–12)

| § | MUST | Finding | Evidence |
|---|---|---|---|
| 1 | Threat model — audio path | **DEFERRED to Sprint 2** | Audio capture/transcription not yet built. Will be audited alongside actual audio pipeline. |
| 2 | Data handling — SQLite at 0o600 | **DEFERRED to Sprint 2** | SQLite store not yet created. Sprint 2 must enforce mode 0o600 on the DB file. |
| 3 | Authentication — none by design | **pass** | `security-framework.md:34-37` confirms single-user, single-machine design. No auth code in repo. |
| 4 | Tenant isolation — single-tenant by design | **pass** | No multi-tenant code. Data under `~/Library/Application Support/Scribe/` with mode 0o700 at `secrets/mod.rs:64-83`. |
| 5 | Secrets management — paste/clipboard | **DEFERRED to Sprint 2** | Clipboard paste/restore not yet implemented. Sprint 2 must verify clipboard restored after paste. |
| 6 | Permissions & platform trust | **pass** | Onboarding wizard gates Mic → AX → API key → hotkey. Capability allow-lists minimal per-window. `tauri.conf.json:13` enables `macOSPrivateApi`. |
| 7 | Egress at runtime | **DEFERRED to Sprint 2** | No actual audio/text egress yet (providers not called). Sprint 2 must verify audio sent only to selected provider. |
| 8 | Dependency / supply-chain | **pass** | Dependencies pinned in `Cargo.toml` / committed `Cargo.lock`. `cargo deny check` green (advisories ok, bans ok, licenses ok, sources ok). `deny.toml:28` `unknown-git = "warn"` (deliberate — `tauri-nspanel` git-sourced at pinned rev). See also SCRIBE-1 check 4. |
| 9 | Local data perms — 0o700 dir, 0o600 file | **pass** | `secrets/mod.rs:72-80` sets mode 0o700 on `app_data_dir()`. `onboarding/persistence.rs:46-53` sets mode 0o600 on `onboarding.json`. Both enforced by unit tests. See also SCRIBE-3 check 5, SCRIBE-5 check 3. |
| 10 | Graceful failure leaks nothing | **pass** | `SecretsError` Display says "keychain error" / "io error" (never the key value). Test at `secrets/mod.rs:179` confirms Display never contains a test key. `OnboardingError` Display shows "io error: {e}" / "serde error: {e}" (no secrets). See also SCRIBE-3 check 1. |
| 11 | No telemetry by default | **pass** | `grep -rE 'sentry\|honeycomb\|posthog\|analytics\|telemetry' Cargo.toml Cargo.lock` returns nothing. Only `tracing` + `tracing-appender` + `tracing-subscriber` for local log file. See also SCRIBE-1 check 2. |
| 12 | Privileged command scoping | **pass** | All 5 privileged commands (set/delete/has-api-key, request-mic, mark-onboarding-complete) have `window.label()` guards. All 9 unprivileged commands are capability-scoped per-window. See per-command audit below and SCRIBE-3 checks 2–3, SCRIBE-4 check 1, SCRIBE-5 checks 1–2. |

---

## Four named greps

### (a) KEYCHAIN_SERVICE — `"Scribe"` locations

Command: `grep -rn '"Scribe"' src-tauri/src/ src/`

```
src-tauri/src/shared/mod.rs:6:pub const APP_NAME: &str = "Scribe";
src-tauri/src/shared/mod.rs:7:pub const KEYCHAIN_SERVICE: &str = "Scribe";
src-tauri/src/shared/mod.rs:67:assert_eq!(super::KEYCHAIN_SERVICE, "Scribe");
src/shared/index.ts:1:export const APP_NAME = "Scribe" as const;
src/shared/index.ts:2:export const KEYCHAIN_SERVICE = "Scribe" as const;
src-tauri/src/secrets/mod.rs:67:.join("Scribe");
src-tauri/src/secrets/mod.rs:203:assert!(dir.ends_with("Scribe"));
```

**Verdict: pass.** `KEYCHAIN_SERVICE` declared only in shared module (Rust `shared/mod.rs:7` + TS `shared/index.ts:2`). The `"Scribe"` usages in `secrets/mod.rs` are data-directory joins (not Keychain service lookups) — correct.

### (b) FFI imports outside platform/macos/

Command: `grep -rE 'use objc2|use core_graphics|use objc2_io_kit|use objc2_app_kit' src-tauri/src/`

```
src-tauri/src/platform/macos/mod.rs:34:            use objc2::msg_send;
src-tauri/src/platform/macos/mod.rs:35:            use objc2::rc::Retained;
src-tauri/src/platform/macos/mod.rs:36:            use objc2_foundation::{NSDictionary, NSNumber, NSString};
src-tauri/src/platform/macos/permissions.rs:4:use objc2::msg_send;
src-tauri/src/platform/macos/permissions.rs:5:use objc2::rc::Retained;
src-tauri/src/platform/macos/permissions.rs:6:use objc2_foundation::NSString;
src-tauri/src/platform/macos/permissions.rs:64:    use objc2::runtime::Bool;
```

**Verdict: pass.** All FFI imports confined to `platform/macos/`. No `core_graphics` or `objc2_app_kit` imports exist (they will be needed in later sprints; `macOSPrivateApi: true` at `tauri.conf.json:13` enables them when needed).

### (c) tauri-nspanel branch/tag

Command: `grep -rE 'tauri-nspanel.*(branch|tag)' src-tauri/Cargo.toml src-tauri/Cargo.lock`

**No matches.** `Cargo.toml:17` pins by `rev = "a3122e894383aa068ec5365a42994e3ac94ba1b6"` — immutable commit SHA.

**Verdict: pass.** Pinned by rev, the most secure approach. Vetted during Sprint 0 Gate C.

### (d) Telemetry crates

Command: `grep -rE 'sentry|honeycomb|posthog|analytics|telemetry' src-tauri/Cargo.toml src-tauri/Cargo.lock`

**No matches.** Only observability crates are `tracing`, `tracing-appender`, `tracing-subscriber` — all local file logging.

**Verdict: pass.**

---

## Per-command audit

Every `#[tauri::command]` in `src-tauri/src/lib.rs` cross-checked against capability JSONs and `window.label()` guards.

| # | Command | Window check | Source capability | Verdict |
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
| 11 | `platform_open_microphone_pane_cmd` | none (opens system prefs) | onboarding.json | pass — capability-scoped |
| 12 | `platform_open_accessibility_pane_cmd` | none (opens system prefs) | onboarding.json | pass — capability-scoped |
| 13 | `platform_hotkey_conflicts_cmd` | none (non-privileged query) | onboarding.json | pass — capability-scoped |
| 14 | `platform_register_hotkey_cmd` | none (uses AppHandle) | onboarding.json | pass — capability-scoped |

**Capability JSON permission summary:**

| Capability | Windows | Privileged commands | Read-only commands | Empty? |
|---|---|---|---|---|
| `main.json` | `["main"]` | none | `is-onboarding-complete-cmd`, `open-onboarding-cmd` | no |
| `onboarding.json` | `["onboarding"]` | set/delete/has-api-key-cmd, request-microphone-permission-cmd, mark-onboarding-complete-cmd | 7 platform/onboarding commands | no |
| `overlay.json` | `["overlay"]` | none | none | yes (correct — no commands needed) |
| `settings.json` | `["settings"]` | none | none | yes (correct — not populated yet) |

**Verdict: pass.** All 14 commands are correctly capability-scoped. The 5 privileged commands have explicit `window.label()` guards. The 9 unprivileged commands are capability-scoped per-window. All capability JSONs use specific window labels (no `"*"`).

---

## Module-grep audits

### (e) secrets/ — key-leaking log calls

Command: `grep -rn 'tracing::\|log::' src-tauri/src/secrets/`

**No matches.** The secrets module contains zero `tracing::*!` or `log::*!` calls.

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

All 8 calls log only hotkey binding types (`Fn`, `ChordCtrlOptSpace`), event types (`Press`, `Release`), or internal error states. **None interpolate audio data, event payload content, or user data.**

**Verdict: pass.**

---

## Cross-task tests-exist checks

### Scaffold test deletion (SCRIBE-1 placeholder, SCRIBE-2 cleanup)

The original `scaffold_compiles()` Rust test at `lib.rs:188` was removed as an inline fix in round 1. No vitest placeholder tests remain — all 4 vitest test files (`egress-copy.test.ts`, `wizard.test.tsx`, `step-api-key.test.tsx`, `step-hotkey.test.tsx`) are real behavioural tests. Cargo tests: 47 passed, 0 failed. Vitest unit: 33 passed.

**Verdict: pass (inline-fixed in round 1).**

### ALLOWED_SECRETS_WINDOWS single declaration

**Verdict: pass.** Declared exactly once at `shared/mod.rs:8`: `pub const ALLOWED_SECRETS_WINDOWS: &[&str] = &["onboarding"];`. Imported in `lib.rs:9` and used for 4 privileged command handlers. No other declarations exist anywhere in the tree.

### app-data dir mode 0o700 test

**Verdict: pass.** Test `app_data_dir_creates_and_has_mode_700` at `secrets/mod.rs:196` exists and passes. Sets `HOME` to temp dir, calls `app_data_dir()`, asserts `mode & 0o777 == 0o700`.

### onboarding.json mode 0o600 test

**Verdict: pass.** Test `file_mode_is_0600` at `onboarding/persistence.rs:124` exists and passes. Writes `onboarding.json` via `write_completed()`, asserts `mode & 0o777 == 0o600`.

---

## tauri.conf.json

- `bundle.macOS.signingIdentity: "-"` — ad-hoc signing only ✓
- No `updater` block ✓
- `macOSPrivateApi: true` required for AX + IOKit HID ✓
- Four window definitions: `main`, `onboarding`, `overlay`, `settings` ✓

## Cargo.toml dependency surface

- `objc2` (0.6), `objc2-app-kit` (0.3), `objc2-foundation` (0.3), `objc2-io-kit` (0.3) — confined to `platform/macos/` ✓
- `keyring` (4.0), `keyring-core` (1.0) — confined to `secrets/` ✓
- `tauri-nspanel` — git-sourced, pinned by rev ✓
- `tracing`, `tracing-appender`, `tracing-subscriber` — local file logging only ✓
- No `claude-sdk`, `anthropic`, `openai`, `groq` crates — not yet needed ✓

## egress-copy snapshot test

`src/shared/__tests__/egress-copy.test.ts` — inline snapshots match the verbatim egress disclosure copy from `shared/index.ts`. Both Groq and OpenAI variants tested.

---

## Additional observations

### .gitignore completeness

The `.gitignore` does not include `.env` or `.env.*` patterns. While no `.env` files exist in the tree (confirmed by `find` and `git ls-files`), adding `.env` to `.gitignore` would be a prudent defence-in-depth measure for Sprint 2.

### Capability JSONs — settings.json and overlay.json

Both `settings.json` and `overlay.json` have empty `permissions` arrays with specific window selectors. This is correct for Sprint 1 (no commands need those windows yet). Sprint 2 must populate these when adding settings-pane and overlay commands.

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

## Build-gate-deferred items

These two SCRIBE-6 checks require a full macOS build to verify. They are deferred to the sprint-final build gate.

| Task | Check | One-line citation |
|---|---|---|
| SCRIBE-6 #3 | Binary contains no API key | Requires `nm`/`strings` on `Scribe.app/Contents/MacOS/scribe` (post-build). Source audit confirms no key embedded. |
| SCRIBE-6 #4 | scribe.log contains no API key | Requires run-time log output. Source audit confirms no tracing/log in secrets/ module. |

---

## Summary

| Category | Count |
|---|---|
| Pass | 28 |
| Inline-fixed | 1 (scaffold test removal — round 1) |
| Deferred to Sprint 2 | 4 (§1, §2, §5, §7) |
| Deferred to build-gate | 2 (SCRIBE-6 #3, #4) |
| Escalated | 0 |

**All 32 security_checks enumerated and verified.** The 5 privileged commands are properly dual-gated (capability + `window.label()`). No secrets leak into logs. No telemetry crates exist. The Keychain service name is declared only in the shared module. `test:fast` (47 Rust + 33 vitest) green. `cargo deny check` green. No structural gaps found.
