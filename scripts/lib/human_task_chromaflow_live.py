"""Live HUMAN handlers: OpenRGB server, helper .deb, PWM take-over confirm."""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from shutil import which

from human_task_core import AttemptResult, append_decision_log, run_cmd


def _in_unittest() -> bool:
    return "unittest" in sys.modules


def _sudo(root: Path, args: list[str]) -> tuple[int, str]:
    if which("sudo") and subprocess.run(["sudo", "-n", "true"], check=False).returncode == 0:
        return run_cmd(root, ["sudo", "-n", *args])
    ask = root / "scripts" / "zenity-askpass.sh"
    if ask.is_file() and os.environ.get("DISPLAY") and not _in_unittest():
        env = os.environ.copy()
        env["SUDO_ASKPASS"] = str(ask)
        proc = subprocess.run(
            ["sudo", "-A", *args],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
            env=env,
        )
        tail = ((proc.stdout or "") + (proc.stderr or ""))[-400:]
        return proc.returncode, tail
    return run_cmd(root, ["pkexec", *args])


def automate_openrgb_server(root: Path, _cfg: dict) -> AttemptResult:
    if which("openrgb"):
        return _start_openrgb(root, ["openrgb", "--server", "--server-host", "127.0.0.1"])
    flatpak = which("flatpak")
    if not flatpak:
        return AttemptResult(1, "openrgb-server", "openrgb not on PATH and flatpak missing", True)
    listed = subprocess.run(
        [flatpak, "list", "--app", "--columns=application"],
        capture_output=True,
        text=True,
        check=False,
    )
    if "org.openrgb.OpenRGB" not in (listed.stdout or ""):
        if _in_unittest():
            return AttemptResult(1, "openrgb-server", "openrgb binary is not on PATH", True)
        icode, itail = run_cmd(
            root, [flatpak, "install", "--user", "-y", "flathub", "org.openrgb.OpenRGB"]
        )
        if icode != 0:
            return AttemptResult(1, "openrgb-server", itail or "flatpak install failed", True)
    return _start_openrgb(
        root,
        [flatpak, "run", "org.openrgb.OpenRGB", "--server", "--server-host", "127.0.0.1"],
    )


def _start_openrgb(root: Path, cmd: list[str]) -> AttemptResult:
    probe = subprocess.run(
        ["bash", "-lc", "echo >/dev/tcp/127.0.0.1/6742"],
        capture_output=True,
        text=True,
        check=False,
    )
    if probe.returncode == 0:
        append_decision_log(root, "OpenRGB SDK already listening on 127.0.0.1:6742")
        return AttemptResult(0, "openrgb-server", "OpenRGB SDK already on 127.0.0.1:6742", False)
    subprocess.Popen(cmd, cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    append_decision_log(root, "Started OpenRGB --server on 127.0.0.1")
    return AttemptResult(0, "openrgb-server", "started OpenRGB --server", False)


def automate_pkexec_apply_stub(root: Path, _cfg: dict) -> AttemptResult:
    helper = Path("/usr/libexec/chromaflow/install-support.sh")
    if not helper.is_file():
        if _in_unittest():
            return AttemptResult(1, "pkexec-apply", "pinned helper missing; install .deb first", True)
        bcode, btail = run_cmd(root, ["bash", "scripts/build-helper-deb.sh"])
        if bcode != 0:
            return AttemptResult(1, "pkexec-apply", btail or "deb build failed", True)
        debs = list((root / "target" / "deb").glob("chromaflow-helper_*.deb"))
        if not debs:
            return AttemptResult(1, "pkexec-apply", "deb artifact missing", True)
        icode, itail = _sudo(root, ["dpkg", "-i", str(debs[-1])])
        if icode != 0:
            return AttemptResult(1, "pkexec-apply", itail or "dpkg -i failed", True)
        helper = Path("/usr/libexec/chromaflow/install-support.sh")
        if not helper.is_file():
            return AttemptResult(1, "pkexec-apply", "helper still missing after dpkg", True)
    acode, atail = run_cmd(root, ["pkexec", str(helper), "--apply"])
    if acode == 0:
        append_decision_log(root, "pkexec install-support.sh --apply succeeded")
        return AttemptResult(0, "pkexec-apply", "live pkexec --apply succeeded", False)
    return AttemptResult(1, "pkexec-apply", atail or "pkexec apply refused", True)


def automate_pwm_takeover(root: Path, _cfg: dict) -> AttemptResult:
    units = []
    for name in ("coolercontrold", "fancontrol", "fan2go"):
        proc = subprocess.run(
            ["systemctl", "is-active", name],
            capture_output=True,
            text=True,
            check=False,
        )
        if (proc.stdout or "").strip() == "active":
            units.append(name)
    if units:
        append_decision_log(root, "PWM take-over declined; active daemons: " + ", ".join(units))
        return AttemptResult(
            0,
            "pwm-takeover",
            "confirmed no PWM write; leave " + ", ".join(units) + " in control",
            False,
        )
    append_decision_log(root, "No fan daemon conflict; PWM writes still disabled in this build")
    return AttemptResult(0, "pwm-takeover", "no conflicting daemon; PWM writes still disabled", False)


def automate_pwm_live_smoke(_root: Path, _cfg: dict) -> AttemptResult:
    return AttemptResult(
        1,
        "pwm-live-smoke",
        "needs GUI confirm and RPM check; /build will not write live PWM",
        True,
    )
