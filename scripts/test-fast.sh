#!/bin/sh
set -eu

vitest run --project unit --project component "$@"
cargo test --manifest-path src-tauri/Cargo.toml --lib
