# Architecture — Scribe

## Tech stack

- **Runtime:** Electron 30+ with TypeScript. The browser-sandbox rules out a web app; only a native local process can hold a global hotkey and inject keystrokes into other apps.
- **Main process build:** `tsup` (esbuild) for fast incremental compiles. Both processes share TypeScript.
- **Renderer framework:** React 18. Settings has meaningful local state; bundle size is irrelevant for a local desktop app.
- **Unified build tool:** `electron-vite` (Vite for renderers, esbuild for main).
- **Packaging:** `electron-builder` producing an unsigned Universal Binary `.app`. No notarization, no auto-updater, no App Store.
- **Local store:** `better-sqlite3` (synchronous, single-writer, no concurrency to manage).
- **Keystroke injection:** `nut.js` (pure-npm, avoids a separate Swift helper). Requires Accessibility permission and is the largest source of build friction.
- **Audio capture:** Web `getUserMedia` + `MediaRecorder` in the overlay renderer; raw PCM accumulated in memory only.
- **Keychain:** `keytar` (one entry per provider, service name `"scribe"`).
- **Global hotkey:** Electron's built-in `globalShortcut`.
- **HTTP client:** Node 18+ built-in `fetch` against the OpenAI-compatible `/v1/audio/transcriptions` endpoint on Groq or OpenAI.
- **Logging:** `electron-log` writing local files only; no remote sinks; explicit lint rule against secret or full-transcript content in log calls.
- **Testing:** `vitest` for unit tests co-located under `src/<module>/__tests__/`; `playwright` Electron mode for integration tests under `tests/`.
- **CI / deployment:** none. Manual rebuild is the update path.

## File and folder structure