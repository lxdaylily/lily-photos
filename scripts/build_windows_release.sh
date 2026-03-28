#!/bin/zsh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="x86_64-pc-windows-gnu"
MINGW_BIN="/opt/homebrew/opt/mingw-w64/bin"

if [[ ! -d "$MINGW_BIN" ]]; then
  echo "MinGW-w64 not found at: $MINGW_BIN" >&2
  echo "Install it first: brew install mingw-w64" >&2
  exit 1
fi

if ! rustup target list --installed | rg -qx "$TARGET"; then
  echo "Rust target not installed: $TARGET" >&2
  echo "Install it first: rustup target add $TARGET" >&2
  exit 1
fi

echo ">> Building Windows release binary"
env PATH="$MINGW_BIN:$PATH" cargo build --release --target "$TARGET" --manifest-path "$ROOT_DIR/Cargo.toml"

echo ">> Windows binary ready:"
echo "$ROOT_DIR/target/$TARGET/release/lily-nest.exe"
