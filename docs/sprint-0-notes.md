# Sprint 0 notes — de-risk spike

Sprint goal: Stand up a runnable throwaway Tauri 2 + Rust spike under `spike/`
and this structured notes skeleton that lets Kees, in a single manual playthrough
on his Mac, return a definite go / no-go on (a) the non-activating NSPanel focus
invariant and (b) the Press / Release detection mechanism for both the `fn` key
and Ctrl+Option+Space — with `tauri-nspanel` pinned and vetted before any FFI
code is written.

Agent-fillable sections (Gate C, Crate versions, Platform trait sketch, the
runbook prose) are filled now. Sections marked `(fill during playthrough)` are
Kees's to write while walking the spike. Sprint verdict at the bottom is the
single go / no-go cell.

## Crate versions

The pins below are the ones Sprint 0 commits to. Task 2 copies them verbatim
into `spike/src-tauri/Cargo.toml`. Sprint 1 carries them into the product repo
unchanged unless a vet here flags an upgrade.

| Crate | Pinned version | Observed during build |
|---|---|---|
| `tauri` | 2.11 | 2.11.2 |
| `tauri-plugin-global-shortcut` | 2.3 | 2.3.2 |
| `objc2` | 0.6 | 0.6.4 |
| `objc2-app-kit` | 0.3 | 0.3.2 |
| `rdev` | 0.5 | 0.5.3 |
| `tauri-nspanel` | git `ahkohd/tauri-nspanel` @ `a3122e894383aa068ec5365a42994e3ac94ba1b6` | git rev `a3122e894383aa068ec5365a42994e3ac94ba1b6` |

The `tauri-nspanel` entry is a git pin, not a crates.io version — the crate is
not published. Task 2's `Cargo.toml` uses
`{ git = "https://github.com/ahkohd/tauri-nspanel", rev = "a3122e894383aa068ec5365a42994e3ac94ba1b6" }`.
That SHA is the contract; do not pin a branch, tag, or `main`.


## Gate C — `tauri-nspanel` vet

Pinned commit: **`a3122e894383aa068ec5365a42994e3ac94ba1b6`** (full 40 character
SHA, branch `v2.1` of github.com/ahkohd/tauri-nspanel, committed 2025-11-19).
Task 2 pins this exact SHA in `spike/src-tauri/Cargo.toml`.

Tauri 2 compatibility verdict: **COMPATIBLE — wrapper crate usable.**

The commit's surface trait is `WebviewWindowExt<R: Runtime>` implemented for
`tauri::WebviewWindow<R>` (`src/lib.rs`), with the conversion method
`FromWindow<R>::from_window(window: WebviewWindow<R>, label: String) -> tauri::Result<Self>`.
That is the Tauri 2 window type — not the Tauri 1 `tauri::Window`. The crate's
own `Cargo.toml` depends on `tauri = { version = "2.8.5", features = ["macos-private-api"] }`,
which confirms Tauri 2 is the supported runtime. The `tauri_panel!` macro
(`src/common.rs`) produces an `NSPanel` subclass that swaps onto the existing
`WebviewWindow`'s `NSWindow`, so the non-activating + floating-level invariants
Sprint 0 cares about can be set against that subclass directly without bypassing
the wrapper.

Dependency-graph note: the commit's `Cargo.toml` pulls in **pure `objc2` 0.6.x —
no legacy `objc` 0.2 anywhere in the dependency graph.** Exact pins inside the
crate: `objc2 = "0.6.1"`, `objc2-app-kit = "0.3.1"`, `objc2-foundation = "0.3.1"`.
These line up with Scribe's pinned `objc2` 0.6 / `objc2-app-kit` 0.3, so no
version juggling is required when Task 2 calls into both the wrapper and direct
`objc2-app-kit` style mask constants from the same crate graph. The macro
infrastructure also re-exports `pastey = "0.2"` for token-paste support — this is
a build-time helper, not a runtime dependency.

Microphone / Accessibility (macOS TCC) surface conclusion: the pinned commit
**does not invoke `AVCaptureDevice`, `AVCaptureSession`, `AXIsProcessTrusted`,
`AXIsProcessTrustedWithOptions`, or `CGEventTap`** anywhere in `src/lib.rs`,
`src/panel.rs`, `src/builder.rs`, `src/common.rs`, or `src/event.rs`. The crate's
job is strictly `NSWindow → NSPanel` subclassing, style mask manipulation,
window level, panel show / hide, and an `NSWindowDelegate` event handler. No
microphone capture, no Accessibility-trust check, no system-wide event tap.
That keeps the supply-chain blast radius narrow: granting Accessibility to the
spike binary is required because of the keystroke-injection path Sprint 0
verifies separately, not because `tauri-nspanel` itself requests it.

Implication for Task 2: pin this SHA; convert the spike's
`tauri::WebviewWindow` to a custom panel class via the `tauri_panel!` macro and
`to_panel::<MyPanel>()`; set `NSWindowStyleMaskNonActivatingPanel` + a floating
level using the wrapper's `set_style_mask` / `set_level` methods (and the
`config: { can_become_key_window: false, can_become_main_window: false }` macro
overrides) rather than reaching into `objc2-app-kit` directly. No FFI bypass
required for Sprint 0.

## Gate A — non-activating NSPanel focus runbook

Goal: confirm that, when the spike's overlay panel appears while TextEdit is the
frontmost app, TextEdit keeps key-window status and any typing while the panel
is visible lands in TextEdit's document, not the panel's webview.

Run:

0. Build the spike: `cargo build --manifest-path spike/src-tauri/Cargo.toml`.
   Launch the resulting binary (Tauri `cargo tauri dev` is fine for Sprint 0;
   release-mode is not required for this gate).
1. Quit any app that owns the global hotkey (other dictation tools, Raycast
   layers, Karabiner-Elements custom maps that bind the same chord). Verify the
   spike's stdout shows `[PLUGIN] registered: fn` and
   `[PLUGIN] registered: Ctrl+Option+Space` before continuing.
2. Open TextEdit (`open -a TextEdit`). Create a new empty Plain Text document
   (Format → Make Plain Text if it opens Rich Text). Click into the document
   body so the insertion caret blinks there.
3. Confirm TextEdit is frontmost by looking at the menu bar — the bold app name
   immediately to the right of the Apple menu must read `TextEdit`. The
   TextEdit document's title-bar chrome must be coloured (key-window state),
   not greyed-out.
4. Trigger the spike's panel via the registered hotkey (fn-key tap, or
   Ctrl+Option+Space if testing the secondary binding). The panel appears.
5. **Wait approximately 1 second for the panel to fully render before typing.**
   The NSPanel subclass is added to the screen via `orderFrontRegardless` (or
   equivalent) which is asynchronous on the main thread; typing during the
   first frame can race the panel becoming visible and produce an inconclusive
   read.
6. Type a short, distinctive phrase (e.g. `hello scribe`). Do not click
   anywhere during or after typing. The panel must remain visible and the
   typing must visibly land in the TextEdit document.
7. Look at the menu bar a second time: it must still read `TextEdit`. Look at
   the TextEdit document title bar: it must still be coloured (key window). The
   panel itself must never show key-window chrome (its title bar — if any — must
   remain greyed-out, since it is non-activating).
8. Dismiss the panel via its normal hide path (escape key, or whatever the
   spike's panel-hide handler binds). Confirm focus is still TextEdit and the
   document contains the phrase you typed.

Pass criteria: menu bar reads TextEdit throughout; TextEdit document receives
the typed phrase verbatim; the panel never steals key-window status. Failure
modes worth recording: menu bar flips to the spike app the moment the panel
appears; typed characters land in the panel's webview console / blackhole
instead of TextEdit; panel renders behind another window (window-level
miscalibration).

Result: **(fill during playthrough)**

## Gate B — Press / Release detection runbook

Goal: confirm that the hotkey provider in use emits both `Press` and `Release`
events for the `fn` key and for `Ctrl+Option+Space`, on this Mac, on this OS
version. Hold-to-talk depends on `Release`; if `Release` never fires, the
dictation flow has no end-of-utterance signal and the spike must fall back to
press-toggle or a different provider.

Run:

0. **After every `cargo build`, re-grant Accessibility to the freshly-built
   spike binary in System Settings → Privacy & Security → Accessibility before
   proceeding.** macOS re-keys the Accessibility trust on the binary's
   code-signing identity / inode, so a rebuild silently invalidates the prior
   grant and `rdev`'s low-level tap goes silent without an error. If you skip
   this step and Gate B fails, the failure is the missing grant, not the
   mechanism.
1. Launch the spike. Watch stdout. The spike registers `fn` and
   `Ctrl+Option+Space` via `tauri-plugin-global-shortcut` (the `[PLUGIN]`
   channel) AND in parallel via `rdev` (the `[RDEV]` channel) so the two
   detection paths can be cross-checked on the same press.
2. Bring TextEdit to the front so the spike does not have key focus during the
   test (Gate A's invariant; reduces the chance the test apparatus itself
   accidentally consumes the key event).
3. For the `fn` binding: press and hold the `fn` key for **approximately 300
   milliseconds — comfortably under macOS's ~500 ms key-repeat threshold** — then
   release. Read stdout for the binding's Press and Release lines on both
   channels.
4. For the `Ctrl+Option+Space` binding: press and hold the chord (control,
   option, space pressed together) for approximately 300 ms, then release. Read
   stdout the same way.
5. Repeat each binding three times to filter one-off jitter. The truth-table
   cell below records the steady-state behaviour, not a single sample.

Dual-log interpretation: **When both `[PLUGIN]` and `[RDEV]` lines fire for the
same binding, the plugin result is canonical; rdev confirms the event reached
the OS.** If the plugin emits Press but no Release while rdev emits both, the
plugin is the broken layer and Sprint 0 may keep the plugin for Press and
switch the Release wiring to rdev. If neither layer emits Release for `fn`, the
event is not observable through the Quartz event stream this spike uses, and
the third truth-table column applies.

Truth table — Gate B results:

| Binding | Plugin emitted Release | `rdev` emitted Release | Not observable via Quartz — IOKit HID required |
|---|---|---|---|
| `fn` | (fill during playthrough) | (fill during playthrough) | (fill during playthrough) |
| `Ctrl+Option+Space` | (fill during playthrough) | (fill during playthrough) | (fill during playthrough) |

Each row's three cells are exclusive: exactly one of the three should be a
positive observation per binding once playthrough completes. If a row records
positive readings in both the plugin and rdev columns, the plugin column wins
(per the canonical rule above) and rdev is logged as confirmation, not as the
chosen path.

## Platform trait sketch

The chosen hotkey mechanism is hidden behind a Platform trait method
`fn register_hotkey(binding: HotkeyBinding, on_event: Box<dyn Fn(HotkeyEvent) + Send + Sync>) -> HotkeyHandle`,
where `HotkeyBinding` carries a modifiers bitfield and a key, `HotkeyEvent` is
`Pressed` or `Released`, and `HotkeyHandle` is the unregistration token. The
macOS implementation may dispatch internally to any of three mechanisms:
(1) `tauri-plugin-global-shortcut` for combos the plugin can register (such as
Ctrl+Option+Space when the plugin path succeeds); (2) `rdev` for modifier-only
keys visible to the Quartz event tap (providing a cross-check channel for any
binding the plugin also handles, and a primary channel if the plugin falls
through); or (3) a direct IOKit HID tap for keys invisible to Quartz (such as
`fn` on Apple Silicon, if Gate B confirms the Quartz path is silent for that
binding). The trait surface does not change across mechanisms; the macOS
implementation dispatches based on the binding shape and what registration
succeeds at startup.

## Architecture-change proposal (fill if Gate A or Gate B fails)

## Sprint verdict

**(fill during playthrough)** — single cell: `GO` if both Gate A and Gate B
pass cleanly on this Mac, otherwise `NO-GO` with a one-line pointer to the
failing gate and the relevant Architecture-change proposal row above.
