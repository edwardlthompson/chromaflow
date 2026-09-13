"""Apply a dry-run support plan. Never writes PWM. Root/pkexec only."""
from __future__ import annotations

import os
import pwd
import re
import subprocess
from pathlib import Path

from chromaflow_extras import EXTRA_IDS, KERNEL_EXTRAS, extra_modprobe, extra_results, extras_status, fan_sio_ok, pending_want, rgb_i2c_ok
from chromaflow_it87 import apply_it87_dkms
from chromaflow_yaml import ALLOWED_APT, ALLOWED_MODULE, load_index

MOD_RE = re.compile(ALLOWED_MODULE)
APT_RE = re.compile(ALLOWED_APT)


def _run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=False)


def _tail(proc: subprocess.CompletedProcess) -> str:
    return ((proc.stderr or proc.stdout or "")[-160:]).strip()


def _append_module(name: str) -> None:
    conf = Path("/etc/modules-load.d/chromaflow.conf")
    lines = conf.read_text(encoding="utf-8").splitlines() if conf.is_file() else []
    if name not in lines:
        conf.write_text("\n".join([*lines, name]) + "\n", encoding="utf-8")


def _usermod(group: str, errors: list[str]) -> None:
    _run(["groupadd", "-f", group])
    uid = os.environ.get("PKEXEC_UID")
    if not (uid and uid.isdigit()):
        return
    try:
        user = pwd.getpwuid(int(uid)).pw_name
        _run(["usermod", "-aG", group, user])
    except KeyError:
        errors.append(f"unknown PKEXEC_UID {uid}")


def apply(data_dir: Path, plan: dict, only: str | None = None) -> dict:
    if os.geteuid() != 0:
        return {"ok": False, "errors": ["apply requires root via pkexec"]}
    if only and only not in EXTRA_IDS:
        return {"ok": False, "errors": [f"unknown extra: {only}"]}
    want = pending_want(only)
    if not want:
        scored = extra_results({only} if only else set(EXTRA_IDS), "")
        return {"ok": True, "apply": True, "installed": [], "loaded": [], "extras": extras_status(), "extra_results": scored, "errors": [], "warnings": [], "pwm": False, "logout_required": True}
    errors: list[str] = []
    warnings: list[str] = []
    installed: list[str] = []
    loaded: list[str] = []
    env = os.environ.copy()
    env["DEBIAN_FRONTEND"] = "noninteractive"
    data = load_index(data_dir)
    optional = {str(row.get("apt") or "") for row in data.get("packages") or [] if row.get("optional_package")}
    apts = [str(a) for a in (plan.get("would_install") or [])]
    if only == "linux-modules-extra":
        apts = [a for a in apts if a.startswith("linux-modules-extra")]
    elif only == "liquidctl":
        apts = ["liquidctl"]
    elif only:
        apts = []
    if "liquidctl" in want and "liquidctl" not in apts:
        apts.append("liquidctl")
    if "liquidctl" not in want:
        apts = [a for a in apts if a != "liquidctl"]
    if "linux-modules-extra" not in want:
        apts = [a for a in apts if not a.startswith("linux-modules-extra")]
    if apts:
        subprocess.run(["apt-get", "update", "-qq"], check=False, capture_output=True, env=env)
    for apt in apts:
        if not APT_RE.fullmatch(apt):
            errors.append(f"illegal apt name: {apt!r}")
            continue
        proc = subprocess.run(["apt-get", "install", "-y", "--no-install-recommends", "--", apt], check=False, capture_output=True, text=True, env=env)
        if proc.returncode == 0:
            installed.append(apt)
            continue
        skip = not only and (apt in optional or apt.startswith("linux-modules-extra"))
        (warnings if skip else errors).append(f"apt {apt}: {_tail(proc)}")
    if "it87-dkms" in want:
        err = apply_it87_dkms()
        errors.append(err) if err else installed.append("it87-dkms")
    if "udev" in want:
        udev = Path("/etc/udev/rules.d/60-chromaflow.rules")
        lines = [str(row.get("line") or "") for row in data.get("udev") or [] if row.get("line")]
        if lines:
            udev.write_text("\n".join(lines) + "\n", encoding="utf-8")
            _run(["udevadm", "control", "--reload-rules"])
            _run(["udevadm", "trigger"])
    names = [extra_modprobe(only)] if only in KERNEL_EXTRAS else ([] if only else [extra_modprobe(i) for i in EXTRA_IDS if i in KERNEL_EXTRAS and i in want])
    names = [n for n in names if MOD_RE.fullmatch(str(n)) and not Path("/sys/module", n.replace("-", "_")).is_dir()]
    if names and only is None:
        Path("/etc/modules-load.d/chromaflow.conf").write_text("\n".join(names) + "\n", encoding="utf-8")
    if names:
        _run(["depmod", "-a"])
    for name in names:
        if only:
            _append_module(name)
        proc = subprocess.run(["modprobe", "--", name], check=False, capture_output=True, text=True)
        if proc.returncode == 0:
            loaded.append(name)
            continue
        msg = f"modprobe {name} failed: {_tail(proc) or 'not present on this kernel'}"
        sibling = (name == extra_modprobe("i2c-nct6775") and rgb_i2c_ok()) or (name == extra_modprobe("nct6775") and fan_sio_ok())
        (warnings if sibling or not (only or extra_modprobe(only or "") == name) else errors).append(msg)
    for g in ("i2c", "plugdev"):
        if f"group_{g}" in want:
            _usermod(g, errors)
    if "pwm_acl" in want:
        acl = Path(os.environ.get("CHROMAFLOW_PWM_ACL", "/usr/libexec/chromaflow/pwm-acl.sh"))
        errors.append("pwm-acl.sh missing") if not acl.is_file() else _run(["bash", str(acl)])
    scored = extra_results(want, "")
    return {"ok": not errors and all(row["ok"] for row in scored), "apply": True, "installed": installed, "loaded": loaded, "extras": extras_status(), "extra_results": scored, "errors": errors, "warnings": warnings, "pwm": False, "logout_required": True}


def main(argv: list[str] | None = None) -> int:
    import json
    import sys

    from chromaflow_support import build_plan

    args = list(sys.argv[1:] if argv is None else argv)
    data_dir = Path(args[0] if args else "/usr/share/chromaflow")
    advanced = "--advanced" in args
    only = args[args.index("--only") + 1] if "--only" in args and args.index("--only") + 1 < len(args) else None
    plan = build_plan(data_dir, advanced=advanced or only in {"i2c-nct6775", "i2c-nvidia-gpu"})
    result = apply(data_dir, plan, only=only)
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
