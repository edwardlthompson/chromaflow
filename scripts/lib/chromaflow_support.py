"""Build a dry-run support plan from YAML allowlists. Never runs apt/modprobe."""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from shutil import which
from subprocess import run

sys.path.insert(0, str(Path(__file__).resolve().parent))
from chromaflow_extras import extras_status
from chromaflow_select import include_safe_module, row_matches
from chromaflow_yaml import ALLOWED_MODULE, load_index

MODULE_RE = re.compile(ALLOWED_MODULE)
APT_RE = re.compile(r"[a-zA-Z0-9._+-]+")


def _probe(env_key: str, fallback: str) -> str:
    override = os.environ.get(env_key)
    if not override:
        return fallback
    path = Path(override)
    return path.read_text(encoding="utf-8", errors="replace") if path.is_file() else override


def _dmi_text() -> str:
    parts = []
    for name in ("board_vendor", "board_name"):
        try:
            parts.append(Path(f"/sys/class/dmi/id/{name}").read_text(encoding="utf-8").strip())
        except OSError:
            continue
    return " ".join(parts)


def _cmd_text(name: str) -> str:
    exe = which(name)
    if not exe:
        return ""
    try:
        proc = run([exe], check=False, capture_output=True, text=True, timeout=3)
        return (proc.stdout or "") + (proc.stderr or "")
    except OSError:
        return ""


def validate_module(row: dict) -> str | None:
    name = str(row.get("modprobe") or "")
    return None if MODULE_RE.fullmatch(name) else f"illegal module name: {name!r}"


def validate_apt(name: str) -> str | None:
    return None if APT_RE.fullmatch(name) else f"illegal apt name: {name!r}"


def build_plan(data_dir: Path, *, advanced: bool) -> dict:
    data = load_index(data_dir)
    blob = "\n".join(
        [
            _probe("CHROMAFLOW_LSPCI", _cmd_text("lspci")),
            _probe("CHROMAFLOW_LSUSB", _cmd_text("lsusb")),
            _probe("CHROMAFLOW_DMI", _dmi_text()),
        ]
    )
    errors: list[str] = []
    would_install: list[str] = []
    warnings: list[str] = []
    for row in data["packages"]:
        apt = str(row.get("apt") or "")
        if row.get("apt_kernel_flavor"):
            apt = f"{apt}-{os.uname().release}"
        err = validate_apt(apt)
        if err:
            errors.append(err)
            continue
        would_install.append(apt)
        if row.get("warning"):
            warnings.append(str(row["warning"]))
    would_load: list[str] = []
    skipped_experimental: list[str] = []
    skipped_not_installed: list[str] = []
    for row in data["modules_safe"]:
        err = validate_module(row)
        if err:
            errors.append(err)
            continue
        name = str(row["modprobe"])
        if not include_safe_module(row, blob):
            continue
        if row.get("require_dpkg"):
            skipped_not_installed.append(name)
            continue
        would_load.append(name)
    for row in data["modules_experimental"]:
        err = validate_module(row)
        if err:
            errors.append(err)
            continue
        name = str(row["modprobe"])
        needles = any(row.get(k) for k in ("match_lspci", "match_lsusb", "match_dmi"))
        take = advanced and name != "nouveau" and (row_matches(row, blob) if needles else True)
        if take:
            would_load.append(name)
        else:
            skipped_experimental.append(name)
    if errors:
        return {"ok": False, "errors": errors}
    return {
        "ok": True,
        "would_install": would_install,
        "would_load": would_load,
        "extras": extras_status(),
        "would_modules_load_d": {
            "path": "/etc/modules-load.d/chromaflow.conf",
            "names": list(would_load),
            "apply": False,
        },
        "skipped_experimental": skipped_experimental,
        "skipped_not_installed": skipped_not_installed,
        "udev_path": "/etc/udev/rules.d/60-chromaflow.rules",
        "logout_required": True,
        "reboot_required": False,
        "warnings": warnings,
        "apply": False,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="ChromaFlow support plan")
    parser.add_argument("--data", required=True)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--advanced", action="store_true")
    args = parser.parse_args(argv)
    plan = build_plan(Path(args.data), advanced=args.advanced)
    if not args.dry_run:
        plan = {**plan, "ok": False, "errors": ["apply is not enabled in this milestone"]}
        print(json.dumps(plan, indent=2))
        return 2
    print(json.dumps(plan, indent=2))
    return 0 if plan.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
