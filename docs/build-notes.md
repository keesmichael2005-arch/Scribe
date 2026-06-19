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

This build uses **ad-hoc signing** (`signingIdentity: "-"` in `tauri.conf.json`).
No Developer ID certificate is used; the binary is not notarised.

Developer ID signing and notarisation for distribution are deferred to Sprint 4.
When that sprint lands, the dmg target will be re-enabled alongside the notarisation
chain.

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
