"""Live extra-kernel checklist. Read-only; never apt/modprobe."""
from __future__ import annotations

import grp
import os
import pwd
from pathlib import Path
from subprocess import run

from chromaflow_it87 import it87_dkms_present

KERNEL_ROWS = (
    ("gigabyte_wmi", "Gigabyte WMI", "gigabyte_wmi"),
    ("it87", "ITE Super I/O fans", "it87"),
    ("nct6775", "Nuvoton Super I/O fans", "nct6775"),
    ("k10temp", "AMD CPU temps", "k10temp"),
    ("i2c-piix4", "AMD SMBus", "i2c-piix4"),
    ("jc42", "DIMM SPD temps", "jc42"),
    ("spd5118", "DDR5 SPD temps", "spd5118"),
    ("i2c-nct6775", "NCT6775 I2C", "nct6775-i2c"),
    ("i2c-nvidia-gpu", "NVIDIA GPU I2C", "i2c-nvidia-gpu"),
)
KERNEL_EXTRAS = {row[0] for row in KERNEL_ROWS} | {"i2c-dev"}
RGB_I2C = frozenset({"i2c-nct6775", "i2c-nvidia-gpu"})
FAN_SIO = frozenset({"it87", "nct6775"})
EXTRA_IDS = (
    "it87-dkms",
    "liquidctl",
    "linux-modules-extra",
    "i2c-dev",
    "i2c-piix4",
    "it87",
    "nct6775",
    "k10temp",
    "jc42",
    "spd5118",
    "gigabyte_wmi",
    "udev",
    "group_i2c",
    "group_plugdev",
    "pwm_acl",
    "i2c-nct6775",
    "i2c-nvidia-gpu",
)


def extra_modprobe(extra_id: str) -> str:
    for eid, _label, mod in KERNEL_ROWS:
        if eid == extra_id:
            return mod
    return extra_id


def _pkg_installed(name: str) -> bool:
    try:
        proc = run(
            ["dpkg-query", "-W", "-f=${Status}", "--", name],
            check=False,
            capture_output=True,
            text=True,
            timeout=3,
        )
    except (OSError, TimeoutError):
        return False
    return proc.returncode == 0 and "install ok installed" in (proc.stdout or "")


def _in_group(group: str) -> bool:
    uid_raw = os.environ.get("PKEXEC_UID")
    try:
        user_uid = int(uid_raw) if uid_raw and uid_raw.isdigit() else os.getuid()
        user = pwd.getpwuid(user_uid).pw_name
    except (KeyError, ValueError):
        user = ""
    try:
        info = grp.getgrnam(group)
    except KeyError:
        return False
    if user in info.gr_mem:
        return True
    if user_uid == os.getuid():
        return info.gr_gid in os.getgroups()
    return False


def _sys_module(name: str) -> bool:
    return Path("/sys/module", name.replace("-", "_")).is_dir()


def _i2c_dev_present() -> bool:
    return _sys_module("i2c_dev") or Path("/sys/class/i2c-dev").is_dir() or any(Path("/dev").glob("i2c-*"))


def rgb_i2c_ok() -> bool:
    return any(_sys_module(extra_modprobe(eid)) for eid in RGB_I2C)


def fan_sio_ok() -> bool:
    return any(_sys_module(extra_modprobe(eid)) for eid in FAN_SIO)


def _modinfo_ok(name: str) -> bool:
    try:
        proc = run(["modinfo", "--", name], check=False, capture_output=True, timeout=3)
    except (OSError, TimeoutError):
        return False
    return proc.returncode == 0


def extras_status() -> list[dict]:
    extra_pkg = f"linux-modules-extra-{os.uname().release}"
    rgb_ok = rgb_i2c_ok()
    sio_ok = fan_sio_ok()
    extra_ok = _pkg_installed(extra_pkg) or _modinfo_ok("nct6775-i2c")
    by = {
        "it87-dkms": {"id": "it87-dkms", "label": "ITE Super I/O DKMS", "present": it87_dkms_present()},
        "liquidctl": {"id": "liquidctl", "label": "liquidctl USB AIO", "present": _pkg_installed("liquidctl") or Path("/usr/bin/liquidctl").is_file()},
        "linux-modules-extra": {"id": "linux-modules-extra", "label": extra_pkg, "present": extra_ok},
        "i2c-dev": {"id": "i2c-dev", "label": "i2c-dev", "present": _i2c_dev_present()},
        "udev": {"id": "udev", "label": "udev hidraw/i2c rules", "present": Path("/etc/udev/rules.d/60-chromaflow.rules").is_file() or Path("/usr/lib/udev/rules.d/60-chromaflow.rules").is_file()},
        "group_i2c": {"id": "group_i2c", "label": "User in i2c", "present": _in_group("i2c")},
        "group_plugdev": {"id": "group_plugdev", "label": "User in plugdev", "present": _in_group("plugdev")},
        "pwm_acl": {"id": "pwm_acl", "label": "PWM sysfs for plugdev", "present": any(bool(p.stat().st_mode & 0o020) for p in Path(os.environ.get("CHROMAFLOW_HWMON_ROOT", "/sys/class/hwmon")).glob("hwmon*/pwm[0-9]") if p.is_file())},
    }
    for eid, label, mod in KERNEL_ROWS:
        present = _sys_module(mod) or (eid in RGB_I2C and rgb_ok) or (eid in FAN_SIO and sio_ok)
        by[eid] = {"id": eid, "label": label, "present": present}
    return [by[i] for i in EXTRA_IDS]


def pending_want(only: str | None) -> set[str]:
    have = {row["id"] for row in extras_status() if row["present"]}
    if only:
        return set() if only in have else {only}
    return {i for i in EXTRA_IDS if i not in have}


def extra_results(wanted: set[str], fallback: str) -> list[dict]:
    out: list[dict] = []
    for row in extras_status():
        if row["id"] not in wanted:
            continue
        if row["present"]:
            out.append({"id": row["id"], "ok": True, "error": ""})
        else:
            msg = fallback or f"{row['id']} is not present on this kernel"
            out.append({"id": row["id"], "ok": False, "error": msg})
    return out
