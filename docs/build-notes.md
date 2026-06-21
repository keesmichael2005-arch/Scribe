# Scribe Build Notes

## Universal Binary Build

Build the ad-hoc-signed Universal `.app` (arm64 + x86_64) from `src-tauri/`:

```bash
cargo tauri build --target universal-apple-darwin --bundles app
```

### Prerequisites

Ensure both Apple Silicon and Intel Rust targets are installed:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

### Output

The build produces `Scribe.app` at:

```
src-tauri/target/universal-apple-darwin/release/bundle/macos/Scribe.app
```

### Verify Universal Binary

Confirm both architecture slices are present:

```bash
lipo -archs src-tauri/target/universal-apple-darwin/release/bundle/macos/Scribe.app/Contents/MacOS/scribe
# Expected output: arm64 x86_64 (order may vary)
```

## Signing

This build uses a **stable self-signed code-signing identity** (`signingIdentity: "Scribe Dev"` in `tauri.conf.json`).
Self-signed is the PERMANENT signing approach for Scribe. Notarisation and Apple Developer-ID
are dropped from the product. Scribe is a personal tool distributed to 2-3 known people;
recipients accept a one-time right-click → Open past Gatekeeper.

### One-time: create the "Scribe Dev" code-signing certificate

1. Open **Keychain Access** (in `/Applications/Utilities`).
2. From the menu bar: **Keychain Access → Certificate Assistant → Create a Certificate**.
3. Fill in the fields:
   - **Name:** `Scribe Dev`
   - **Identity Type:** Self Signed Root
   - **Certificate Type:** Code Signing
   - Check **"Let me override defaults"**
4. Click **Continue** and follow the prompts (accept the default validity period).
5. Verify the cert appears under `My Certificates` in Keychain Access and that it lists under:
   ```bash
   security find-identity -v -p codesigning
   ```
   Look for `"Scribe Dev"` in the output.

The cert is **build-machine-only**. Only the machine that signs builds needs it;
users receive an already-signed `.app` and just grant TCC permissions once.

### Why self-signed

macOS TCC binds Accessibility, Input Monitoring, and Microphone grants to the
app's code identity (its designated requirement, derived from the signing cert +
bundle id). Ad-hoc signing (`"-"`) gives no stable designated requirement — each
rebuild produces a different identity, so TCC grants do not bind to a prior build's
identity and the running binary stays Denied. A single stable self-signed cert used
for every build gives one constant designated requirement; TCC grants then bind to
the running binary and persist across rebuilds.

## Security validation (post-build)

With the built `.app`, verify the binary contains no leaked API key:

```bash
# nm check — must return nothing
nm src-tauri/target/universal-apple-darwin/release/bundle/macos/Scribe.app/Contents/MacOS/scribe | grep -F '<test API key prefix>'

# strings check — must return nothing
strings src-tauri/target/universal-apple-darwin/release/bundle/macos/Scribe.app/Contents/MacOS/scribe | grep -F '<test API key prefix>'
```

And confirm `~/Library/Logs/Scribe/scribe.log` does not contain the key:

```bash
grep -r '<test API key prefix>' ~/Library/Logs/Scribe/ 2>/dev/null
# must return nothing
```

## Build date

Last successful build: _(populated after the Universal build completes on Apple Silicon)_
