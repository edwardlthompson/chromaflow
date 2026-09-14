#!/usr/bin/env bash
# Print the product semver from CHANGELOG.md (not .template-version).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ver=""
while IFS= read -r line; do
  case "$line" in
    "## ["[0-9]*.[0-9]*.[0-9]*"]"*)
      ver="${line#*\[}"
      ver="${ver%%]*}"
      break
      ;;
  esac
done < "$ROOT/CHANGELOG.md"
if [ -z "$ver" ]; then
  echo "FAIL: CHANGELOG.md has no [X.Y.Z] product version" >&2
  exit 1
fi
printf '%s\n' "$ver"
