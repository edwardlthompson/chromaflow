#!/usr/bin/env bash
# Hash ChromaFlow NSIS installers and rewrite packaging/winget/chromaflow/*.installer.yaml.
# Does not submit to microsoft/winget-pkgs. [HUMAN] opens that PR after URLs match Release.
# Usage:
#   WINGET_INSTALLER_X64=dist/ChromaFlow_0.2.2_x64-setup.exe \
#   WINGET_INSTALLER_ARM64=dist/ChromaFlow_0.2.2_arm64-setup.exe \
#     bash scripts/publish-winget-hashes.sh
# Dry-run (packs tiny placeholders when assets missing):
#   bash scripts/publish-winget-hashes.sh --dry-run
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# shellcheck source=lib/resolve-python.sh
. "$(cd "$(dirname "$0")" && pwd)/lib/resolve-python.sh"

INSTALLER_YAML="$ROOT/packaging/winget/chromaflow/EdwardThompson.ChromaFlow.installer.yaml"
OUT_DIR="${WINGET_HASH_OUT:-$ROOT/dist/winget-hashes}"
DRY=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
  esac
done

mkdir -p "$OUT_DIR"
X64="${WINGET_INSTALLER_X64:-}"
ARM64="${WINGET_INSTALLER_ARM64:-}"

if [ "$DRY" = "1" ]; then
  if [ -z "$X64" ] || [ ! -f "$X64" ]; then
    X64="$OUT_DIR/ChromaFlow_dry_x64-setup.exe"
    printf "ChromaFlow winget dry-run x64\n" > "$X64"
  fi
  if [ -z "$ARM64" ] || [ ! -f "$ARM64" ]; then
    ARM64="$OUT_DIR/ChromaFlow_dry_arm64-setup.exe"
    printf "ChromaFlow winget dry-run arm64\n" > "$ARM64"
  fi
  echo "NOTE dry-run using $X64 and $ARM64"
fi

if [ -z "$X64" ] || [ ! -f "$X64" ]; then
  echo "FAIL: set WINGET_INSTALLER_X64 to an existing installer, or pass --dry-run"
  exit 1
fi
if [ -z "$ARM64" ] || [ ! -f "$ARM64" ]; then
  echo "FAIL: set WINGET_INSTALLER_ARM64 to an existing installer, or pass --dry-run"
  exit 1
fi

"$PY" - <<PY
import hashlib
import re
from pathlib import Path

def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()

x64 = Path(r"""$X64""")
arm64 = Path(r"""$ARM64""")
dest = Path(r"""$INSTALLER_YAML""")
text = dest.read_text(encoding="utf-8")
hx = sha256(x64)
ha = sha256(arm64)

def replace_arch(body: str, arch: str, digest: str) -> str:
    pattern = rf"(Architecture: {arch}\n(?:    .*\n)*?    InstallerSha256: )([0-9a-fA-F]+)"
    new, n = re.subn(pattern, rf"\g<1>{digest}", body, count=1)
    if n != 1:
        raise SystemExit(f"FAIL: could not patch InstallerSha256 for {arch}")
    return new

text = replace_arch(text, "x64", hx)
text = replace_arch(text, "arm64", ha)
dest.write_text(text, encoding="utf-8")
print(f"Updated {dest}")
print(f"  x64    {hx}")
print(f"  arm64  {ha}")
PY

echo "OK hashed winget installer manifest → $INSTALLER_YAML"
echo "[HUMAN] Confirm InstallerUrl points at GitHub Release HTTPS assets before microsoft/winget-pkgs"
