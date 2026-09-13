"""Frank Crawford it87 DKMS extra. Never bundles .ko. Never silent 0%."""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

PKG = "it87-dkms"
CONF = Path("/etc/modprobe.d/chromaflow-it87.conf")
LIVE = Path("/sys/module/it87/srcversion")
OPT = "options it87 ignore_resource_conflict=1\n"
MISSING = "it87-dkms is not in apt; install frankcrawford it87 DKMS (do not add CoolerControl automatically)"


def _pkg_ok() -> bool:
    try:
        proc = subprocess.run(
            ["dpkg-query", "-W", "-f=${Status}", "--", PKG],
            check=False,
            capture_output=True,
            text=True,
            timeout=3,
        )
    except (OSError, TimeoutError):
        return False
    return proc.returncode == 0 and "install ok installed" in (proc.stdout or "")


def dkms_ko() -> Path | None:
    root = Path("/lib/modules", os.uname().release, "updates", "dkms")
    for name in ("it87.ko.zst", "it87.ko.xz", "it87.ko"):
        path = root / name
        if path.is_file():
            return path
    return None


def srcversion(path: Path) -> str:
    try:
        proc = subprocess.run(
            ["modinfo", "-F", "srcversion", "--", str(path)],
            check=False,
            capture_output=True,
            text=True,
            timeout=3,
        )
    except (OSError, TimeoutError):
        return ""
    return (proc.stdout or "").strip()


def it87_dkms_present() -> bool:
    ko = dkms_ko()
    if not ko or not LIVE.is_file():
        return False
    want = srcversion(ko)
    return bool(want) and LIVE.read_text(encoding="utf-8").strip() == want


def apply_it87_dkms() -> str | None:
    if os.geteuid() != 0:
        return "it87-dkms apply requires root via pkexec"
    if it87_dkms_present():
        if not CONF.is_file():
            CONF.write_text(OPT, encoding="utf-8")
        return None
    env = os.environ.copy()
    env["DEBIAN_FRONTEND"] = "noninteractive"
    if not _pkg_ok():
        subprocess.run(["apt-get", "update", "-qq"], check=False, capture_output=True, env=env)
        proc = subprocess.run(
            ["apt-get", "install", "-y", "--no-install-recommends", "--", PKG],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )
        if proc.returncode != 0:
            return MISSING
    CONF.write_text(OPT, encoding="utf-8")
    subprocess.run(["modprobe", "-r", "it87"], check=False, capture_output=True)
    proc = subprocess.run(
        ["modprobe", "it87", "ignore_resource_conflict=1"],
        check=False,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return f"modprobe it87 failed: {(proc.stderr or proc.stdout or '')[-160:].strip()}"
    if not it87_dkms_present():
        return "it87 loaded but srcversion is not the DKMS module"
    return None
