#!/usr/bin/env bash
# Restore owned PWM to firmware (pwm*_enable=2). Never writes duty. Never enable 0.
set -euo pipefail
ROOT="${CHROMAFLOW_HWMON_ROOT:-/sys/class/hwmon}"
OWNED="${CHROMAFLOW_OWNED:-${XDG_RUNTIME_DIR:-/tmp}/chromaflow/owned}"
n=0
if [[ -f "$OWNED" ]]; then
  while read -r chip pwm _; do
    [[ "$chip" =~ ^hwmon[0-9]+$ ]] || continue
    [[ "$pwm" =~ ^pwm[0-9]+$ ]] || continue
    en="$ROOT/$chip/${pwm}_enable"
    [[ -f "$en" ]] || continue
    printf '2\n' > "$en" || continue
    n=$((n + 1))
  done < "$OWNED"
fi
echo "chromaflow pwm-failsafe: restored pwm*_enable=2 (firmware) on $n channel(s), never 0. apply=true"
exit 0
