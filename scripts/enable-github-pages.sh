#!/usr/bin/env bash
# Enable GitHub Pages (Actions source) and dispatch pages.yml. Idempotent.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=lib/resolve-python.sh
. "$(cd "$(dirname "$0")" && pwd)/lib/resolve-python.sh"
exec "$PY" "$ROOT/scripts/lib/github_pages_enable.py" "$@"
