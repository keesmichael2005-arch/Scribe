# Build Order — Scribe, blueprint v1

Total sprints: 4

## Sequence

1. **Scaffold, build pipeline, native module smoke, onboarding gates 1-3** — Stand up the Electron + TypeScript app shell with electron-vite and electron-builder configured for a Universal Binary, prove that both native modules (nut.js and better-sqlite3) compile against the target Electron version end-to-end, install the Accessibility trust startup check, ship the menu bar presence, build the keychain module, and deliver onboarding wizard steps 1-3 (Microphone, Accessibility, API key entry with egress disclosure). After this sprint the app launches, presents a menu bar icon, walks the owner through three of four permission/key gates, and has demonstrated that the native module pipeline can ship as a Universal Binary.
   Depends on: none
   Surface: desktop-shell
2. **Core dictation pipeline end-to-end with failure states** — Wire the full speak-to-paste loop: global hotkey activation, microphone capture with waveform IPC, STT call against Groq or OpenAI, clipboard save / nut.js keystroke / clipboard restore, and the non-focusing pill overlay with all four states (idle, listening, transcribing, error). Ship graceful degradation for offline, provider error, and paste-blocked field in the same sprint that ships the pipeline (per rules-and-constraints.md Must rule). Build the storage module (transcripts + settings + retention job + file permission tighten) so failed rows are persisted for later retry. Close onboarding step 4 (test recording) by wiring it through the new pipeline. After this sprint the owner can begin daily use.
   Depends on: 1
   Surface: desktop-pipeline
3. **Settings window with full configuration surface and retry** — Ship the Settings BrowserWindow with every owner-configurable surface: hotkey remapping with live re-registration, capture mode toggle, provider selection, model selection per provider, API key management per provider (Keychain-backed), a test transcription button that does not paste, a recent transcripts list with retry action for failed rows, and the retention window control. This sprint makes accent accuracy tunable — the owner's stated second reason for building Scribe — and gives the owner a recovery path for any transcript that failed in sprint 2.
   Depends on: 1, 2
   Surface: desktop-settings
4. **Edge case sweep, regression invariants, Universal Binary release** — Run the full play-tester playbook against every flow and edge case in user-flow-testing.md, lock in every regression invariant as an automated test, harden every error path against the security-framework.md MUST-check list, and produce a working unsigned Universal Binary .app that opens via right-click → Open on a clean macOS install. After this sprint the app is shippable as a personal-use artifact and re-buildable on demand.
   Depends on: 1, 2, 3
   Surface: release-hardening

## Dependency graph

- (root) → 1. Scaffold, build pipeline, native module smoke, onboarding gates 1-3
- 1. Scaffold, build pipeline, native module smoke, onboarding gates 1-3 → 2. Core dictation pipeline end-to-end with failure states
- 1. Scaffold, build pipeline, native module smoke, onboarding gates 1-3 + 2. Core dictation pipeline end-to-end with failure states → 3. Settings window with full configuration surface and retry
- 1. Scaffold, build pipeline, native module smoke, onboarding gates 1-3 + 2. Core dictation pipeline end-to-end with failure states + 3. Settings window with full configuration surface and retry → 4. Edge case sweep, regression invariants, Universal Binary release

## Sequencing rationale

The dominant sequencing constraint is that nothing useful happens until Microphone permission, Accessibility permission, and a provider API key are all in place. That makes the first sprint heavy on permission gates and onboarding scaffolding, not on the dictation loop itself. The Technical voice raised a valid objection to the Product voice's "ship onboarding first then pipeline" framing: onboarding step 4 (test recording) depends on a working pipeline, which means step 4 cannot close inside the onboarding sprint. The synthesised cut is to ship onboarding steps 1-3 in the scaffold sprint (these depend only on `keychain/` and the Electron shell), and treat step 4 as the integration acceptance gate that closes the pipeline sprint.

Sprint 1 also has to prove the native module pipeline (`nut.js` + `better-sqlite3` as Universal Binary against the chosen Electron version). If that build is broken, the entire injection approach needs reconsidering before pipeline code is written. Combining this with the `keychain/` module and onboarding steps 1-3 keeps the sprint coherent and clears the risk early.

Sprint 2 wires the full pipeline end-to-end. The Product voice was emphatic and the Wiki voice agreed: the pipeline must ship with its failure states (offline banner, provider-error banner, paste-blocked clean failure, "Transcribing..." linger). Graceful degradation is a Must rule in `rules-and-constraints.md`, not polish. Silent failures during a daily-use tool's first days will destroy trust before the tool has proven itself. The retention job lives here as well because `storage/` is built in this sprint. The retry UI for failed rows is deferred to the Settings sprint because it requires the Settings BrowserWindow.

Sprint 3 ships Settings. The defaults (Groq, tap-to-toggle, fn key) are usable from sprint 2 onward, but Settings is what makes accent accuracy tunable, which is the owner's stated second reason for building Scribe.

Sprint 4 is hardening and packaging in a single pass: edge case sweep against `user-flow-testing.md`, regression invariant verification, and Universal Binary build. Dev mode has been the daily-use surface since sprint 2; packaging is needed for a stable install path, not for first value delivery.
