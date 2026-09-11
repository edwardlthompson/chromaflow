"""install-support.sh --dry-run must not mutate the system."""
from __future__ import annotations

import json
import os
import stat
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "install-support.sh"


class SupportDryRunTests(unittest.TestCase):
    def test_dry_run_json_allowlist(self) -> None:
        trap = self._trap_dir()
        env = os.environ.copy()
        env["PATH"] = f"{trap}{os.pathsep}{env.get('PATH', '')}"
        env["CHROMAFLOW_LSPCI"] = "Intel SMBus\nAMD VGA"
        env["CHROMAFLOW_LSUSB"] = ""
        env["CHROMAFLOW_DMI"] = "ASUS ROG"
        proc = subprocess.run(
            ["bash", str(SCRIPT), "--dry-run"],
            check=False,
            capture_output=True,
            text=True,
            env=env,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        plan = json.loads(proc.stdout)
        self.assertTrue(plan["ok"])
        self.assertFalse(plan["apply"])
        self.assertTrue(plan["logout_required"])
        for name in plan["would_load"]:
            self.assertRegex(name, r"^[a-zA-Z0-9_-]+$")
        self.assertIn("nct6775", plan["would_load"])
        self.assertIn("i2c-nvidia-gpu", plan["skipped_experimental"])
        self.assertIn("zenpower", plan["skipped_not_installed"])
        self.assertTrue((trap / "apt-get.ran").read_text() == "")
        self.assertTrue((trap / "modprobe.ran").read_text() == "")

    def test_apply_refuses_from_repo(self) -> None:
        proc = subprocess.run(
            ["bash", str(SCRIPT), "--apply"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertNotEqual(proc.returncode, 0)
        blob = proc.stderr + proc.stdout
        self.assertIn("pkexec", blob)
        env = os.environ.copy()
        env["PKEXEC_UID"] = "1000"
        env["CHROMAFLOW_POLKIT"] = "1"
        proc2 = subprocess.run(
            ["bash", str(SCRIPT), "--apply"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            env=env,
        )
        self.assertNotEqual(proc2.returncode, 0)
        self.assertIn("pinned path", proc2.stderr + proc2.stdout)

    def test_rejects_illegal_module_names(self) -> None:
        import re

        cre = re.compile(r"^[a-zA-Z0-9_-]+$")
        self.assertIsNone(cre.fullmatch("../../x"))
        self.assertIsNone(cre.fullmatch("nvidia-drm;reboot"))
        self.assertIsNone(cre.fullmatch(""))
        self.assertTrue(cre.fullmatch("nct6775"))

    def _trap_dir(self) -> Path:
        trap = ROOT / "target" / "support-trap"
        trap.mkdir(parents=True, exist_ok=True)
        for name in ("apt-get", "modprobe", "usermod", "udevadm"):
            path = trap / name
            path.write_text(
                f"#!/bin/sh\necho ran >> \"{trap / (name + '.ran')}\"\nexit 42\n",
                encoding="utf-8",
            )
            path.chmod(path.stat().st_mode | stat.S_IEXEC)
            (trap / f"{name}.ran").write_text("", encoding="utf-8")
        return trap


if __name__ == "__main__":
    unittest.main()
