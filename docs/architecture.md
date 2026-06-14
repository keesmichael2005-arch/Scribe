# Architecture — Scribe

## Tech stack

- **Shell:** Tauri 2 (Rust core + macOS WKWebView). Not Electron, not a localhost web app — a sandboxed browser tab cannot register a system-wide hotkey or inject keystrokes into other apps. Brief and prior wiki D2 lock this.
- **Frontend:** TypeScript + React 18 + Vite. Recorded as wiki D3. Bundle size irrelevant for a local desktop app.
- **Global hotkey:** `tauri-plugin-global-shortcut` primary; `rdev` or `CGEventTap` via `objc2`/`core-graphics` as fallback if the plugin does not emit key-up. Sprint 0 resolves which path.
- **Audio capture:** `cpal` (pure-Rust CoreAudio binding). PCM accumulated in memory; WAV encoded in-memory at stop time. No `std::fs` call ever receives audio.
- **STT HTTP client:** `reqwest` against the OpenAI-compatible `/v1/audio/transcriptions` endpoint. Groq and OpenAI both implement it; `stt` presents one interface over both.
- **Keystroke injection:** `enigo` primary (wraps CGEvent); `CGEventPost` via `objc2`/`core-graphics` for finer control. Requires Accessibility permission — startup `AXIsProcessTrustedWithOptions` check is mandatory.
- **Clipboard:** `arboard` or `tauri-plugin-clipboard-manager`. Save + set + paste + restore; transcript text never lingers on clipboard after paste completes.
- **Overlay window:** `tauri-nspanel` (git: `ahkohd/tauri-nspanel`, **pinned to a specific commit hash and vetted in Sprint 0**) + `objc2` FFI for `NSWindowStyleMaskNonActivatingPanel` + floating window level. Transparent, always-on-top, decorationless, non-focusable.
- **Local storage:** SQLite via `rusqlite` (synchronous, single-writer). `:memory:` for unit tests; `~/Library/Application Support/Scribe/scribe.db` with `0600` mode in production.
- **Secrets:** `keyring` crate, macOS Keychain backend, service name `"Scribe"`, one entry per provider. Never in repo, config, DB, or logs.
- **Packaging:** Tauri bundler, Universal binary (`aarch64-apple-darwin` + `x86_64-apple-darwin`), signed + notarized `.app` / `.dmg`. Build path validated in Sprint 1, cut in Sprint 4.
- **Logging:** Rust `tracing` / `log` to `~/Library/Logs/Scribe/`. No remote sinks. Explicit rule: no secrets, no full transcript text in log calls.
- **Testing:** Rust `cargo test --lib`; frontend `vitest run` (never bare `vitest`). Unit + component only — no integration, no pgTAP, no e2e. The native system behaviours are verified by manual playthrough.
- **Supply-chain:** `cargo deny` configured in Sprint 1; `cargo audit` runs in Sprint 4 before the signed build is cut.

**Crate version pins** (from prior wiki architecture.md, carried forward): `tauri 2.11`, `tauri-plugin-global-shortcut 2.3`, `tauri-plugin-sql 2.4`, `tauri-plugin-clipboard-manager 2.3`, `cpal 0.17`, `reqwest 0.13`, `enigo 0.6`, `arboard 3.6`, `keyring 4.0`, `rusqlite 0.40`, `objc2`/`objc2-app-kit` 0.6/0.3, `tauri-nspanel` pinned commit hash.

## File and folder structure