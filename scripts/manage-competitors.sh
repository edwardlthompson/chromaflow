#!/usr/bin/env bash
# Stop or uninstall competing fan/RGB daemons. Default is --dry-run. No PWM.
set -euo pipefail

PINNED="/usr/libexec/chromaflow/manage-competitors.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
RESOLVED="$SCRIPT_PATH"
if command -v realpath >/dev/null 2>&1; then
  RESOLVED="$(realpath "$SCRIPT_PATH")"
fi

if [ "$RESOLVED" = "$PINNED" ]; then
  LIB_DIR="/usr/libexec/chromaflow"
else
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  LIB_DIR="$ROOT/scripts/lib"
fi

usage() {
  cat <<'EOF'
Usage: manage-competitors.sh [--dry-run] [--remove-apply]
  --dry-run       Print JSON uninstall plan. Default. Never apt/systemctl.
  --remove-apply  pkexec + pinned path only. Stops units and apt-get purge allowlisted packages.
EOF
}

MODE="dry-run"
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --remove-apply) MODE="remove-apply"; shift ;;
    --stop-apply) MODE="stop-apply"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

refuse_apply() {
  if [ -z "${PKEXEC_UID:-}" ]; then
    echo '{"ok":false,"errors":["apply requires pkexec (PKEXEC_UID and CHROMAFLOW_POLKIT=1)"]}' >&2
    return 1
  fi
  CHROMAFLOW_POLKIT="${CHROMAFLOW_POLKIT:-1}"
  export CHROMAFLOW_POLKIT
  if [ "$RESOLVED" != "$PINNED" ]; then
    echo '{"ok":false,"errors":["apply requires pinned path /usr/libexec/chromaflow/manage-competitors.sh"]}' >&2
    return 1
  fi
  return 0
}

if [ "$MODE" = "remove-apply" ]; then
  refuse_apply || exit 1
  python3 "$LIB_DIR/chromaflow_competitors.py" --remove --apply
  exit $?
fi
if [ "$MODE" = "stop-apply" ]; then
  refuse_apply || exit 1
  python3 "$LIB_DIR/chromaflow_competitors.py" --stop --apply
  exit $?
fi

export PATH="${CHROMAFLOW_SAFE_PATH:-$PATH}"
python3 "$LIB_DIR/chromaflow_competitors.py" --remove
