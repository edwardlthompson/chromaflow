#!/usr/bin/env bash
# Download the pinned OpenRGB AppImage (GPL sibling). Not committed.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PIN="$ROOT/data/openrgb-engine.yaml"
OUT="${CHROMAFLOW_ENGINE_CACHE:-$ROOT/target/engine/OpenRGB.AppImage}"
mkdir -p "$(dirname "$OUT")"
eval "$(python3 - "$PIN" <<'PY'
import sys
from pathlib import Path
vals = {}
for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if ":" not in line or line.lstrip().startswith("#"):
        continue
    key, _, rest = line.partition(":")
    vals[key.strip()] = rest.strip()
for name in ("host", "url", "sha256", "max_bytes"):
    if not vals.get(name):
        raise SystemExit(f"missing {name} in pin")
print(f"HOST={vals['host']}")
print(f"URL={vals['url']}")
print(f"SHA={vals['sha256'].lower()}")
print(f"MAX={vals['max_bytes']}")
PY
)"
rest="${URL#https://}"
got="${rest%%/*}"
if [ "$got" != "$HOST" ]; then
  echo "engine URL host is not allowlisted" >&2
  exit 2
fi
if [ -f "$OUT" ]; then
  got_sha="$(sha256sum "$OUT" | awk '{print $1}')"
  if [ "$got_sha" = "$SHA" ]; then
    echo "$OUT"
    exit 0
  fi
fi
part="$OUT.part"
rm -f "$part"
curl -fL --proto =https --tlsv1.2 --max-time 120 --max-filesize "$MAX" -o "$part" "$URL"
got_sha="$(sha256sum "$part" | awk '{print $1}')"
if [ "$got_sha" != "$SHA" ]; then
  rm -f "$part"
  echo "engine sha256 mismatch" >&2
  exit 2
fi
chmod 0755 "$part"
mv -f "$part" "$OUT"
echo "$OUT"
