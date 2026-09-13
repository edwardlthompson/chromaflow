"""Plan or apply uninstall of competing fan/RGB daemons. Never writes PWM."""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

UNITS = {
    "fancontrol": {"units": ["fancontrol"], "packages": ["fancontrol"]},
    "coolercontrold": {
        "units": ["coolercontrold", "coolercontrol-liqctld"],
        "packages": ["coolercontrol", "coolercontrold", "coolercontrol-liqctld"],
    },
    "fan2go": {"units": ["fan2go"], "packages": ["fan2go"]},
    "thinkfan": {"units": ["thinkfan"], "packages": ["thinkfan"]},
    "nbfc_service": {"units": ["nbfc_service"], "packages": ["nbfc-linux"]},
    "openrazer-daemon": {
        "units": ["openrazer-daemon"],
        "packages": ["openrazer-daemon", "openrazer-driver-dkms", "polychromatic"],
    },
    "ckb-next-daemon": {"units": ["ckb-next-daemon"], "packages": ["ckb-next"]},
}
UNIT_RE = re.compile(r"^[a-z][a-z0-9_-]*$")
PKG_RE = re.compile(r"^[a-z0-9][a-z0-9.+-]*$")
KNOWN = frozenset(UNITS)
ALLOWED_UNITS = {u for r in UNITS.values() for u in r["units"]}
ALLOWED_PKGS = {p for r in UNITS.values() for p in r["packages"]}


def _pidof(name: str) -> bool:
    return subprocess.run(["pidof", "--", name], check=False, capture_output=True).returncode == 0


def _unit_present(path: Path) -> bool:
    if path.is_symlink() and str(path.readlink()) == "/dev/null":
        return False
    return path.is_file()


def _unit_live(name: str) -> bool:
    active = subprocess.run(
        ["systemctl", "is-active", "--quiet", "--", f"{name}.service"],
        check=False,
        capture_output=True,
    )
    if active.returncode == 0:
        return True
    enabled = subprocess.run(
        ["systemctl", "is-enabled", "--quiet", "--", f"{name}.service"],
        check=False,
        capture_output=True,
    )
    return enabled.returncode == 0


def detect() -> list[str]:
    raw = os.environ.get("CHROMAFLOW_CONFLICTS")
    if raw is not None:
        return [n for n in raw.split(",") if n.strip() in KNOWN]
    found: list[str] = []
    for name, row in UNITS.items():
        if any(_unit_live(u) or _pidof(u) for u in row["units"]):
            found.append(name)
    return found


def plan_for(names: list[str], action: str = "remove") -> dict:
    units: list[str] = []
    packages: list[str] = []
    for name in names:
        row = UNITS.get(name)
        if not row:
            continue
        units.extend(u for u in row["units"] if UNIT_RE.fullmatch(u))
        packages.extend(p for p in row["packages"] if PKG_RE.fullmatch(p))
    uniq = lambda xs: list(dict.fromkeys(xs))
    act = action if action in ("stop", "remove") else "remove"
    return {
        "ok": True,
        "apply": False,
        "pwm": False,
        "action": act,
        "detected": [n for n in names if n in KNOWN],
        "would_stop": uniq(units),
        "would_remove": uniq(packages) if act == "remove" else [],
        "errors": [],
    }


def apply(plan: dict) -> dict:
    if os.geteuid() != 0:
        return {"ok": False, "apply": False, "pwm": False, "errors": ["apply requires root via pkexec"]}
    errors: list[str] = []
    stopped: list[str] = []
    removed: list[str] = []
    env = {**os.environ, "DEBIAN_FRONTEND": "noninteractive"}
    for unit in plan.get("would_stop") or []:
        if not UNIT_RE.fullmatch(str(unit)) or str(unit) not in ALLOWED_UNITS:
            errors.append(f"illegal unit: {unit!r}")
            continue
        subprocess.run(["systemctl", "disable", "--now", "--", f"{unit}.service"], check=False)
        subprocess.run(["pkill", "-x", "--", str(unit)], check=False)
        stopped.append(str(unit))
    for pkg in plan.get("would_remove") or []:
        if not PKG_RE.fullmatch(str(pkg)) or str(pkg) not in ALLOWED_PKGS:
            errors.append(f"illegal package: {pkg!r}")
            continue
        proc = subprocess.run(
            ["apt-get", "remove", "--purge", "-y", "--", str(pkg)],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )
        if proc.returncode == 0:
            removed.append(str(pkg))
        else:
            errors.append(f"apt {pkg}: {(proc.stderr or proc.stdout or '')[-120:]}")
    return {
        "ok": not errors,
        "apply": True,
        "pwm": False,
        "stopped": stopped,
        "removed": removed,
        "errors": errors,
    }


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    action = "stop" if "--stop" in args else "remove"
    do_apply = "--apply" in args
    plan = plan_for(detect(), action=action)
    if do_apply:
        result = apply(plan)
        print(json.dumps(result, indent=2))
        return 0 if result.get("ok") else 1
    print(json.dumps(plan, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
