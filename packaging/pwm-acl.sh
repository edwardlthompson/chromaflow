#!/usr/bin/env bash
# Grant plugdev group-write on hwmon pwm* / pwm*_enable. Never writes duty.
set -euo pipefail
ROOT="${CHROMAFLOW_HWMON_ROOT:-/sys/class/hwmon}"
shopt -s nullglob
n=0
for f in "$ROOT"/hwmon*/pwm*; do
  base="$(basename "$f")"
  [[ "$base" =~ ^pwm[0-9]+(_enable)?$ ]] || continue
  chgrp plugdev "$f" 2>/dev/null || true
  chmod 0660 "$f" || true
  n=$((n + 1))
done
echo "chromaflow pwm-acl: plugdev 0660 on $n pwm nodes" >&2
exit 0
