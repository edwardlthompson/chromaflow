#!/bin/sh
# Enable login autostart and a Cinnamon panel favorite. No PWM. No root required.
set -e
systemctl --user daemon-reload >/dev/null 2>&1 || true
systemctl --user enable chromaflowd.service >/dev/null 2>&1 || true
systemctl --user enable chromaflow-sdk.service >/dev/null 2>&1 || true
systemctl --user enable chromaflow-gui.service >/dev/null 2>&1 || true
if command -v gsettings >/dev/null 2>&1; then
  python3 - <<'PY' || true
import ast
import subprocess

key = ["gsettings", "get", "org.cinnamon", "favorite-apps"]
try:
    raw = subprocess.check_output(key, text=True).strip()
    apps = list(ast.literal_eval(raw))
except Exception:
    raise SystemExit(0)
desk = "chromaflow.desktop"
if desk not in apps:
    apps.insert(0, desk)
    encoded = "[" + ", ".join("'" + a.replace("'", "") + "'" for a in apps) + "]"
    subprocess.check_call(["gsettings", "set", "org.cinnamon", "favorite-apps", encoded])
PY
fi
exit 0
