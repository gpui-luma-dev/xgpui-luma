#!/bin/sh
# Run from any directory. Requires the WASM Rust target and matching wasm-bindgen CLI.
set -eu
cd "$(dirname "$0")/../.."
profile="${1:-release}"
case "$profile" in
  release) directory=release ;;
  dev) directory=debug ;;
  *) echo "Usage: sh apps/docs-embed/build.sh [release|dev]" >&2; exit 2 ;;
esac
# Native linker flags (for example, -lc++) are not valid for the WASM target.
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
cargo build --locked -p luma-docs-embed --target wasm32-unknown-unknown --profile "$profile"
wasm-bindgen --target web --out-dir apps/docs-embed/pkg \
  "target/wasm32-unknown-unknown/$directory/luma_docs_embed.wasm"
python3 -B apps/docs-embed/package_runtime.py
