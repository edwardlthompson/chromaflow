#!/usr/bin/env bash
# Shared product smoke for Linux and Windows CI (same steps on both).
# Usage: bash scripts/smoke-chromaflow.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== chromaflow smoke: cargo test (exclude desktop) ==="
cargo test --workspace --exclude chromaflow-desktop

echo "=== chromaflow smoke: CLI support ==="
cargo run -p chromaflow-cli -- support

echo "=== chromaflow smoke: desktop npm build ==="
(
  cd apps/desktop
  if [ ! -d node_modules ]; then
    npm ci
  fi
  npm run build
)

echo "OK chromaflow smoke"