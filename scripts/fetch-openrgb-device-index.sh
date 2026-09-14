#!/usr/bin/env bash
# Shallow-clone pinned OpenRGB and write detector identity CSV. Never vendors C++.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PIN="$ROOT/data/openrgb-engine.yaml"
OUT="${CHROMAFLOW_DEVICE_INDEX:-$ROOT/data/openrgb-device-index.csv}"
SRC="${CHROMAFLOW_OPENRGB_SRC:-}"
eval "$(python3 - "$PIN" <<'PY'
from pathlib import Path
import sys
vals = {}
for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if ":" not in line or line.lstrip().startswith("#"):
        continue
    key, _, rest = line.partition(":")
    vals[key.strip()] = rest.strip().strip('"')
rev = vals.get("revision") or ""
url = vals.get("git") or "https://codeberg.org/OpenRGB/OpenRGB.git"
if not rev:
    raise SystemExit("missing revision in openrgb-engine.yaml")
print(f"REV={rev}")
print(f"GIT={url}")
PY
)"
if [ -z "$SRC" ]; then
  SRC="$(mktemp -d /tmp/openrgb-index.XXXXXX)"
  trap 'rm -rf "$SRC"' EXIT
  git clone --depth 1 --filter=blob:none "$GIT" "$SRC"
  git -C "$SRC" fetch --depth 1 origin "$REV"
  git -C "$SRC" checkout --detach "$REV"
fi
export CHROMAFLOW_OPENRGB_SRC="$SRC"
export CHROMAFLOW_DEVICE_INDEX="$OUT"
export CHROMAFLOW_OPENRGB_REV="$REV"
python3 "$ROOT/scripts/lib/openrgb_device_index.py"
