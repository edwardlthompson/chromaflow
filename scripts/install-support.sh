#!/usr/bin/env bash
# ChromaFlow detection support helper. Default is --dry-run (no mutation).
set -euo pipefail

PINNED="/usr/libexec/chromaflow/install-support.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
RESOLVED="$SCRIPT_PATH"
if command -v realpath >/dev/null 2>&1; then
  RESOLVED="$(realpath "$SCRIPT_PATH")"
fi

if [ "$RESOLVED" = "$PINNED" ]; then
  ROOT="/usr/share/chromaflow"
  DATA_DIR="${CHROMAFLOW_DATA:-$ROOT}"
  LIB_DIR="/usr/libexec/chromaflow"
else
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  DATA_DIR="${CHROMAFLOW_DATA:-$ROOT/data}"
  LIB_DIR="$ROOT/scripts/lib"
fi

usage() {
  cat <<'EOF'
Usage: install-support.sh [--dry-run] [--advanced] [--apply] [--only NAME]
  --dry-run   Print JSON plan. Default. Never apt/modprobe/udev.
  --advanced  Include experimental allowlist rows in would_load.
  --apply     pkexec + pinned /usr/libexec path only. Installs packages, udev, groups, modules.
  --only NAME One extra id (it87-dkms, liquidctl, linux-modules-extra, i2c-dev, i2c-piix4, it87, nct6775, k10temp, jc42, spd5118, gigabyte_wmi, udev, group_i2c, group_plugdev, pwm_acl, i2c-nct6775, i2c-nvidia-gpu).
EOF
}

MODE="dry-run"
ADVANCED=()
ONLY_ARGS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) MODE="dry-run"; shift ;;
    --apply) MODE="apply"; shift ;;
    --advanced) ADVANCED+=(--advanced); shift ;;
    --only)
      case "${2:-}" in
        it87-dkms|liquidctl|linux-modules-extra|i2c-dev|i2c-piix4|it87|nct6775|k10temp|jc42|spd5118|gigabyte_wmi|udev|group_i2c|group_plugdev|pwm_acl|i2c-nct6775|i2c-nvidia-gpu)
          ONLY_ARGS=(--only "$2"); shift 2 ;;
        *) echo '{"ok":false,"errors":["unknown extra"]}' >&2; exit 2 ;;
      esac
      ;;
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
    echo '{"ok":false,"errors":["apply requires pinned path /usr/libexec/chromaflow/install-support.sh"]}' >&2
    return 1
  fi
  return 0
}

if [ "$MODE" = "apply" ]; then
  refuse_apply || exit 1
  python3 "$LIB_DIR/chromaflow_apply.py" "$DATA_DIR" "${ADVANCED[@]}" "${ONLY_ARGS[@]}"
  exit $?
fi

# Dry-run: Python planner only. PATH may contain stubs that fail if invoked.
export PATH="${CHROMAFLOW_SAFE_PATH:-$PATH}"
python3 "$LIB_DIR/chromaflow_support.py" --data "$DATA_DIR" --dry-run "${ADVANCED[@]}"
