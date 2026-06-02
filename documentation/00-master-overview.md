# Product blueprint — Scribe

## North star

Press a key, talk into any text field on the Mac, and have the exact words land where the cursor is — instantly, privately, and (in later versions) getting better at how the user actually speaks. v1 advances three of those four axes: instantly (single-pass pipeline, no second-stage LLM), privately (local-first, ephemeral audio, Keychain secrets, single-provider egress), and everywhere (global hotkey, press-time target capture across monitors and apps). The "gets better" axis is explicitly a v2 concern.

"We won" 12 months out means Kees never touches the keyboard to enter voice content in any field, the NZ accent is transcribed accurately enough that he stops correcting obvious errors, and the clipboard paste-with-restore loop is trustworthy enough for sensitive input.

## In scope (this blueprint)

- Global hotkey activation (default fn key, fully remappable) for any focused field on any monitor in any app.
- Two capture modes: hold-to-talk and tap-to-toggle, switchable in Settings.
- Capture pill HUD with live waveform that does not steal focus from the target field.
- "Transcribing..." pill linger state with animated dots from key release until text lands (no silent gap).
- Swappable STT provider slot behind an OpenAI-compatible interface. Groq (`whisper-large-v3-turbo` default, `whisper-large-v3` alternate) and OpenAI (`gpt-4o-transcribe`, `gpt-4o-mini-transcribe`).
- Clipboard save / set / paste-with-`nut.js` / restore delivery. No keystroke-typing fallback.
- Local SQLite store for transcripts and settings with 7-day default retention (configurable) and a delete job.
- macOS Keychain for API keys (one entry per provider).
- First-run onboarding wizard, four gated steps: Microphone permission, Accessibility permission, API key entry (with plain-English egress disclosure), test recording.
- Settings window: hotkey remap, mode toggle, provider + model picker, key management, test transcription, recent transcripts list with retry action for failed rows, retention window setting.
- Menu bar presence with distinct icon states (idle / listening / sending / done / error).
- Graceful failure handling: offline banner, provider-error banner, paste-blocked clean failure. Failed transcripts persisted for recovery.
- Universal Binary `.app` packaging (Intel + Apple Silicon), unsigned, right-click-Open install path.

## User surfaces

There is one user role (the owner). The distinct surfaces are:

- **Capture / dictation HUD** — sees that Scribe is listening and receives the transcript in the right field without flow-break.
- **Menu bar presence** — sees app state at a glance and reaches Settings without window-switching.
- **Onboarding wizard** — completes the four required setup steps in a single guided sequence.
- **Settings window** — configures every aspect (hotkey, mode, provider, model, keys, transcripts, retention) from one place.

A v2 cleanup-and-learning layer is acknowledged as a future surface but is not part of this blueprint.

## Feature inventory

**Capture / dictation HUD**
- Global hotkey activation, fn default, remappable.
- Hold-to-talk and tap-to-toggle capture modes.
- Pill with live waveform while listening, no focus theft.
- "Transcribing..." pill state with animated dots from key release until text lands.
- Paste into the press-time target field, multi-monitor correct.
- Clipboard save + restore around every paste.
- Transient error / offline banner in the pill area for failure states.
- Failed transcripts persisted with retry available from Settings.

**Menu bar presence**
- Icon with discrete states: idle / listening / sending / done / error.
- "Open Settings" menu item.

**Onboarding wizard (four gated steps)**
- Step 1: Microphone permission with plain-English explanation.
- Step 2: Accessibility permission with plain-English explanation and a note that re-granting may be needed after rebuilds.
- Step 3: API key entry for at least one provider, with the egress disclosure copy ("Your audio is sent to [Groq / OpenAI] for transcription. No processing happens on this device.").
- Step 4: Test recording with transcript preview, no paste.

**Settings window**
- Hotkey remap.
- Capture mode toggle.
- Provider selection (Groq / OpenAI).
- Model selection per provider.
- API key management per provider (Keychain-backed).
- Test transcription (records, transcribes, displays, does not paste).
- Recent transcripts list with retry action for failed rows.
- Retention window setting (default 7 days, configurable).

## Sequencing rationale

The dominant sequencing constraint is that nothing useful happens until Microphone permission, Accessibility permission, and a provider API key are all in place. That makes the first sprint heavy on permission gates and onboarding scaffolding, not on the dictation loop itself. The Technical voice raised a valid objection to the Product voice's "ship onboarding first then pipeline" framing: onboarding step 4 (test recording) depends on a working pipeline, which means step 4 cannot close inside the onboarding sprint. The synthesised cut is to ship onboarding steps 1-3 in the scaffold sprint (these depend only on `keychain/` and the Electron shell), and treat step 4 as the integration acceptance gate that closes the pipeline sprint.

Sprint 1 also has to prove the native module pipeline (`nut.js` + `better-sqlite3` as Universal Binary against the chosen Electron version). If that build is broken, the entire injection approach needs reconsidering before pipeline code is written. Combining this with the `keychain/` module and onboarding steps 1-3 keeps the sprint coherent and clears the risk early.

Sprint 2 wires the full pipeline end-to-end. The Product voice was emphatic and the Wiki voice agreed: the pipeline must ship with its failure states (offline banner, provider-error banner, paste-blocked clean failure, "Transcribing..." linger). Graceful degradation is a Must rule in `rules-and-constraints.md`, not polish. Silent failures during a daily-use tool's first days will destroy trust before the tool has proven itself. The retention job lives here as well because `storage/` is built in this sprint. The retry UI for failed rows is deferred to the Settings sprint because it requires the Settings BrowserWindow.

Sprint 3 ships Settings. The defaults (Groq, tap-to-toggle, fn key) are usable from sprint 2 onward, but Settings is what makes accent accuracy tunable, which is the owner's stated second reason for building Scribe.

Sprint 4 is hardening and packaging in a single pass: edge case sweep against `user-flow-testing.md`, regression invariant verification, and Universal Binary build. Dev mode has been the daily-use surface since sprint 2; packaging is needed for a stable install path, not for first value delivery.

## Out of scope

- v2 Gemini cleanup pass (single-pass is a v1 invariant; reintroducing a second stage changes the latency and egress profile).
- v2 correction / learning layer and personal glossary.
- Static vocabulary / custom prompt (deferred per reverse-prompt context).
- Keystroke-typing fallback (must-not for v1; clean failure preferred).
- Spend guard / usage cap.
- Auto-updater, code signing, notarization, App Store (no distribution, manual rebuild is the update path).
- Language auto-detection (hard-coded `en`).
- Accounts, auth, cloud database, multi-user, web version (permanently out).

## Risks (cross-cutting)

- **nut.js Accessibility trust revoked on every dev rebuild.** macOS hashes the binary digest; every rebuild silently revokes Accessibility trust, breaking paste. Mitigation: an `isTrustedAccessibilityClient(false)` startup check in sprint 1 that surfaces a clear modal rather than letting paste fail mysteriously at runtime. The onboarding wizard step 2 copy must explicitly note that re-granting may be required after software changes.
- **Universal Binary build complexity with two native modules.** `nut.js` and `better-sqlite3` both require native compilation across arm64 + x86_64. Mitigation: validate the full Universal Binary build in sprint 1 before any pipeline code is written.
- **Capture pill stealing focus from the paste target.** A misconfigured BrowserWindow takes keyboard focus and breaks the entire UX silently. Mitigation: hard-locked window flags (`focusable: false`, `type: 'panel'`, `alwaysOnTop: true` at `'floating'` level), reinforced by a regression invariant that the previously-focused app retains focus after `showOverlay()`.
- **Audio IPC volume on long dictations.** Streaming raw PCM chunks renderer-to-main at 44.1 kHz produces tens of MB per minute. Mitigation: stream only downsampled waveform samples to the overlay (~60/sec); accumulate the full buffer in one process and send it across at stop time as a single IPC call.
- **Provider key leak via naive error logging.** HTTP error responses can echo back `Authorization: Bearer <key>`. Mitigation: never log raw HTTP error objects; the STT module extracts `status`, `statusText`, and a sanitised message only.
- **Onboarding cliff.** Three sequential gates before the first dictation: Mic, Accessibility (fragile, re-grant-prone), and a paid API key on a provider billed separately from any ChatGPT/Claude subscription. Each failure mode needs plain-English guidance matching the branding voice.

## Open questions

The council surfaced four open items. None block the design; all four block clean agent execution and should be resolved in the synthesis pass before the first sprint task fires.

- **OHQ non-negotiables placeholder in `wiki/projects/scribe/rules-and-constraints.md`.** Both Wiki and Product voices flagged it. The current playbook entries are Supabase/database patterns that do not apply to a local desktop app. Recommended resolution: either paste verbatim from `wiki/system/playbook.md` or explicitly mark inapplicable with a one-line rationale.
- **Blue hex, typography, exact pill dimensions, waveform style — all TBD in `branding.md`.** Agents building the pill and Settings UI will invent values otherwise. Recommended resolution: lock a single clean modern blue palette and document it as part of the sprint that ships the pill, with a binding back into `branding.md`.
- **Default provider vs accent goal.** Groq is the recorded default (D4a) and the reverse-prompt context confirmed it. The narrower live question is whether the onboarding wizard step 4 should recommend switching to OpenAI `gpt-4o-transcribe` if the test recording quality is poor. Recommended resolution: do not auto-recommend a switch in v1; surface both the transcript and a one-line note that "Settings lets you switch to OpenAI for better accent accuracy at higher cost." The owner picks.
- **Renderer framework.** React 18 is the proposed default. The Technical voice raised Preact / vanilla as alternatives. Given the Settings window's state surface (hotkey recorder, key forms with validation, transcript list with retry, two provider pickers, mode toggle, retention config) React is the right call. Closed as React 18.

Honest disclosure about debate depth: neither subagent voice independently explored the actual repo (it is a one-commit greenfield with a single-sentence README and no `package.json`). All architectural claims here are derived from the wiki docs and the reverse-prompt context, not from inspection of existing code. The voices are aligned on every load-bearing decision; the remaining divergence is the sprint-shape question resolved in the sequencing rationale.