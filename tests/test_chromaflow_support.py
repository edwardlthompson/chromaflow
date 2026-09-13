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
        env["CHROMAFLOW_LOADED_MODULES"] = "nct6775"
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
        self.assertIn("i2c-dev", plan["would_load"])
        self.assertIn("jc42", plan["would_load"])
        self.assertIn("spd5118", plan["would_load"])
        self.assertIn("asus-wmi", plan["would_load"])
        self.assertIn("i2c-i801", plan["would_load"])
        self.assertNotIn("it8686", plan["would_load"])
        self.assertIn("i2c-nvidia-gpu", plan["skipped_experimental"])
        self.assertEqual(plan["would_modules_load_d"]["path"], "/etc/modules-load.d/chromaflow.conf")
        self.assertFalse(plan["would_modules_load_d"]["apply"])
        self.assertEqual(plan["would_modules_load_d"]["names"], plan["would_load"])
        ids = [row["id"] for row in plan["extras"]]
        self.assertEqual(ids[0], "it87-dkms")
        self.assertEqual(ids[1], "liquidctl")
        self.assertIn("linux-modules-extra", ids)
        self.assertIn("i2c-dev", ids)
        self.assertIn("it87", ids)
        self.assertIn("nct6775", ids)
        self.assertIn("k10temp", ids)
        self.assertIn("i2c-piix4", ids)
        self.assertIn("jc42", ids)
        self.assertIn("spd5118", ids)
        self.assertIn("i2c-nvidia-gpu", ids)
        self.assertTrue((trap / "apt-get.ran").read_text() == "")
        self.assertTrue((trap / "modprobe.ran").read_text() == "")

    def test_unmatched_superio_stays_out_unless_loaded(self) -> None:
        trap = self._trap_dir()
        env = os.environ.copy()
        env["PATH"] = f"{trap}{os.pathsep}{env.get('PATH', '')}"
        env["CHROMAFLOW_LSPCI"] = "Intel SMBus\nAMD VGA"
        env["CHROMAFLOW_LSUSB"] = ""
        env["CHROMAFLOW_DMI"] = "ASUS ROG"
        env["CHROMAFLOW_LOADED_MODULES"] = ""
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
        self.assertNotIn("nct6775", plan["would_load"])
        self.assertIn("i2c-dev", plan["would_load"])
        self.assertIn("asus-wmi", plan["would_load"])
        self.assertNotIn("zenpower", plan["skipped_not_installed"])

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

    def test_policy_and_apply_are_one_click(self) -> None:
        policy = (ROOT / "packaging/polkit/org.chromaflow.install-support.policy").read_text(encoding="utf-8")
        self.assertIn("org.chromaflow.install-support.advanced", policy)
        self.assertIn("--advanced", policy)
        apply = (ROOT / "scripts/lib/chromaflow_apply.py").read_text(encoding="utf-8")
        self.assertIn("optional_package", apply)
        self.assertIn("linux-modules-extra", apply)
        self.assertIn("groupadd", apply)
        self.assertIn("usermod", apply)
        self.assertIn('"i2c"', apply)
        self.assertIn('"plugdev"', apply)
        self.assertNotIn("set_pwm", apply)
        rust = (ROOT / "crates/chromaflow-core/src/support.rs").read_text(encoding="utf-8")
        self.assertIn("pkexec", rust)
        self.assertIn("--apply", rust)
        self.assertIn("--only", rust)
        self.assertIn("json_from_output", rust)
        self.assertIn("auth_admin_keep", policy)
        self.assertIn("on:all={() => runApply()}", (ROOT / "apps/desktop/src/pages/Support.svelte").read_text(encoding="utf-8"))
        extras = (ROOT / "scripts/lib/chromaflow_extras.py").read_text(encoding="utf-8")
        self.assertIn("/usr/lib/udev/rules.d/60-chromaflow.rules", extras)
        self.assertIn("i2c-dev", extras)
        self.assertIn("/sys/class/i2c-dev", extras)
        self.assertIn("nct6775-i2c", extras)
        self.assertIn("pwm_acl", extras)
        self.assertIn("pwm-acl.sh", apply)
        self.assertIn('extra_results(want, "")', apply)
        self.assertIn(">&2", (ROOT / "packaging/pwm-acl.sh").read_text(encoding="utf-8"))
        self.assertIn("pwm_acl", (ROOT / "scripts" / "install-support.sh").read_text(encoding="utf-8"))
        self.assertIn("extra_results", apply)
        self.assertIn("--only", (ROOT / "scripts" / "install-support.sh").read_text(encoding="utf-8"))
        self.assertIn("chromaflow_extras.py", (ROOT / "scripts/build-helper-deb.sh").read_text(encoding="utf-8"))
        self.assertIn("chromaflow_it87.py", (ROOT / "scripts/build-helper-deb.sh").read_text(encoding="utf-8"))
        self.assertIn("it87-dkms", apply)
        self.assertIn("pending_want", apply)
        self.assertIn("liquidctl", apply)
        self.assertIn("it87-dkms", (ROOT / "scripts" / "install-support.sh").read_text(encoding="utf-8"))

    def test_unknown_extra_is_rejected_before_pkexec(self) -> None:
        proc = subprocess.run(
            ["bash", str(SCRIPT), "--apply", "--only", "nouveau"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("unknown extra", proc.stderr + proc.stdout)
        self.assertNotIn("set_pwm", (ROOT / "scripts/lib/chromaflow_apply.py").read_text(encoding="utf-8"))

    def test_nct6775_i2c_maps_and_rgb_sibling_counts_present(self) -> None:
        import sys

        sys.path.insert(0, str(ROOT / "scripts" / "lib"))
        import chromaflow_extras as extras

        self.assertEqual(extras.extra_modprobe("i2c-nct6775"), "nct6775-i2c")
        self.assertEqual(extras.extra_modprobe("i2c-dev"), "i2c-dev")
        orig = extras._sys_module

        def fake(name: str) -> bool:
            return name.replace("-", "_") == "i2c_nvidia_gpu"

        extras._sys_module = fake  # type: ignore[method-assign]
        try:
            by = {row["id"]: row["present"] for row in extras.extras_status()}
            self.assertTrue(by["i2c-nvidia-gpu"])
            self.assertTrue(by["i2c-nct6775"])
            self.assertTrue(extras.rgb_i2c_ok())
        finally:
            extras._sys_module = orig

        def fake_sio(name: str) -> bool:
            return name.replace("-", "_") == "it87"

        extras._sys_module = fake_sio  # type: ignore[method-assign]
        try:
            by = {row["id"]: row["present"] for row in extras.extras_status()}
            self.assertTrue(by["it87"])
            self.assertTrue(by["nct6775"])
            self.assertTrue(extras.fan_sio_ok())
        finally:
            extras._sys_module = orig
        orig_pkg = extras._pkg_installed
        orig_mod = extras._modinfo_ok
        extras._pkg_installed = lambda _n: False  # type: ignore[method-assign]
        extras._modinfo_ok = lambda n: n == "nct6775-i2c"  # type: ignore[method-assign]
        try:
            by = {row["id"]: row["present"] for row in extras.extras_status()}
            self.assertTrue(by["linux-modules-extra"])
        finally:
            extras._pkg_installed = orig_pkg
            extras._modinfo_ok = orig_mod
        js = (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8")
        for eid in extras.EXTRA_IDS:
            self.assertIn(f'"{eid}"', js)
        yaml = (ROOT / "data/modules-experimental.yaml").read_text(encoding="utf-8")
        self.assertIn("modprobe: nct6775-i2c", yaml)
        sh = (ROOT / "scripts" / "install-support.sh").read_text(encoding="utf-8")
        self.assertIn("it87|nct6775|k10temp", sh)
        rust = (ROOT / "crates/chromaflow-core/src/support.rs").read_text(encoding="utf-8")
        self.assertIn('"it87"', rust)
        self.assertIn('"it87-dkms"', rust)
        locales = json.loads((ROOT / "apps/desktop/src/locales/en.json").read_text(encoding="utf-8"))
        self.assertIn("lighting.extras.it87", locales)
        self.assertIn("lighting.extras.it87-dkms", locales)
        self.assertIn("lighting.extras.nct6775", locales)
        self.assertIn("lighting.extras.liquidctl", locales)
        self.assertNotIn("set_pwm", (ROOT / "scripts/lib/chromaflow_extras.py").read_text(encoding="utf-8"))
        self.assertNotIn("it87-dkms", extras.KERNEL_EXTRAS)
        self.assertEqual(extras.EXTRA_IDS[0], "it87-dkms")
        self.assertEqual(extras.EXTRA_IDS[1], "liquidctl")
        have = {row["id"] for row in extras.extras_status() if row["present"]}
        self.assertTrue(extras.pending_want(None).isdisjoint(have))
        if "liquidctl" in have:
            self.assertEqual(extras.pending_want("liquidctl"), set())
        else:
            self.assertEqual(extras.pending_want("liquidctl"), {"liquidctl"})

    def test_it87_dkms_present_is_srcversion_not_in_tree(self) -> None:
        import sys
        import tempfile

        sys.path.insert(0, str(ROOT / "scripts" / "lib"))
        import chromaflow_it87 as it87

        self.assertIsNotNone(it87.apply_it87_dkms())
        self.assertIn("pkexec", str(it87.apply_it87_dkms()))
        src = (ROOT / "scripts/lib/chromaflow_it87.py").read_text(encoding="utf-8")
        self.assertNotIn("set_pwm", src)
        self.assertNotIn("apt-add-repository", src)
        self.assertNotIn("coolercontrol.org", src)
        self.assertIn("frankcrawford", src)
        with tempfile.TemporaryDirectory(prefix="cf-it87-") as raw:
            live = Path(raw) / "srcversion"
            live.write_text("DKMSHASH\n", encoding="utf-8")
            ko = Path(raw) / "it87.ko"
            ko.write_bytes(b"x")
            orig_live, orig_ko, orig_ver = it87.LIVE, it87.dkms_ko, it87.srcversion
            it87.LIVE = live
            it87.dkms_ko = lambda: ko  # type: ignore[method-assign]
            it87.srcversion = lambda _p: "DKMSHASH"  # type: ignore[method-assign]
            try:
                self.assertTrue(it87.it87_dkms_present())
                it87.srcversion = lambda _p: "INTREE"  # type: ignore[method-assign]
                self.assertFalse(it87.it87_dkms_present())
            finally:
                it87.LIVE, it87.dkms_ko, it87.srcversion = orig_live, orig_ko, orig_ver

    def test_rejects_illegal_module_names(self) -> None:
        import re

        cre = re.compile(r"^[a-zA-Z0-9_-]+$")
        self.assertIsNone(cre.fullmatch("../../x"))
        self.assertIsNone(cre.fullmatch("nvidia-drm;reboot"))
        self.assertIsNone(cre.fullmatch(""))
        self.assertTrue(cre.fullmatch("nct6775"))

    def test_device_form_redacts_home_and_omits_serial(self) -> None:
        proc = subprocess.run(
            [
                "node",
                "--input-type=module",
                "-e",
                """
import { buildMarkdown, issueForm } from './apps/desktop/src/lib/deviceReport.js';
const row = {
  name: 'SteelSeries Prime Neo Noir',
  vendor_id: '1038',
  product_id: '1856',
  hid_name: 'SteelSeries Prime',
  kind: 'mouse',
  path: '/home/edward/Repo/hidraw2',
  readable: true,
  reason: 'no_linux_backend',
  manufacturer: 'SteelSeries',
  product: 'Prime Neo',
};
const md = buildMarkdown(row, {
  notes: 'user@example.com and /home/edward/secret.env',
  kernel: '7.0.0-31-generic',
  home: '/home/edward',
});
if (md.includes('serial:')) throw new Error('serial present');
if (md.includes('/home/edward')) throw new Error('home leaked');
if (md.includes('user@example.com')) throw new Error('email leaked');
if (md.includes('secret.env')) throw new Error('env leaked');
const long = issueForm('x'.repeat(4000), '[device]: Prime');
if (!long.bodyTooLarge) throw new Error('expected long body');
if (!long.url.includes('template=device.yml')) throw new Error('template');
if (long.url.includes('report=')) throw new Error('report in long url');
console.log('ok');
""",
            ],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        yml = (ROOT / ".github" / "ISSUE_TEMPLATE" / "device.yml").read_text(encoding="utf-8")
        self.assertIn("[device]:", yml)
        self.assertIn("LLM01", yml)
        self.assertNotIn("set_pwm", yml)
        research = (ROOT / "apps/desktop/src/lib/ResearchList.svelte").read_text(encoding="utf-8")
        report = (ROOT / "apps/desktop/src/lib/deviceReport.js").read_text(encoding="utf-8")
        self.assertIn("submitDeviceReport", research)
        self.assertIn("lighting.reportConfirm", research)
        self.assertIn("markReported", report)
        self.assertIn("label:device", report)
        self.assertNotIn("includeSerial", research)
        page = (ROOT / "apps/desktop/src/pages/Support.svelte").read_text(encoding="utf-8")
        self.assertNotIn("deviceForm", page)
        self.assertNotIn("set_pwm", page)

    def test_competitors_uninstall_plan_is_dry_run(self) -> None:
        script = ROOT / "scripts" / "manage-competitors.sh"
        trap = self._trap_dir()
        env = os.environ.copy()
        env["PATH"] = f"{trap}{os.pathsep}{env.get('PATH', '')}"
        env["CHROMAFLOW_CONFLICTS"] = "fancontrol,coolercontrold,openrazer-daemon"
        proc = subprocess.run(
            ["bash", str(script), "--dry-run"],
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
        self.assertFalse(plan["pwm"])
        self.assertIn("fancontrol", plan["detected"])
        self.assertIn("fancontrol", plan["would_remove"])
        self.assertIn("coolercontrol", plan["would_remove"])
        self.assertIn("openrazer-daemon", plan["would_remove"])
        self.assertNotIn("openrgb", plan["would_remove"])
        self.assertTrue((trap / "apt-get.ran").read_text() == "")
        refused = subprocess.run(
            ["bash", str(script), "--remove-apply"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertNotEqual(refused.returncode, 0)
        self.assertIn("pkexec", refused.stderr + refused.stdout)
        py = (ROOT / "scripts/lib/chromaflow_competitors.py").read_text(encoding="utf-8")
        self.assertNotIn("set_pwm", py)
        self.assertNotIn('"openrgb"', py)
        self.assertNotIn('"chromaflow"', py)
        import sys

        sys.path.insert(0, str(ROOT / "scripts" / "lib"))
        from chromaflow_competitors import _unit_present

        masked = ROOT / "target" / "coolercontrold-mask-test.service"
        masked.parent.mkdir(parents=True, exist_ok=True)
        if masked.exists() or masked.is_symlink():
            masked.unlink()
        masked.symlink_to("/dev/null")
        self.assertFalse(_unit_present(masked))
        masked.unlink()
        policy = (ROOT / "packaging/polkit/org.chromaflow.competitors.policy").read_text(encoding="utf-8")
        self.assertIn("--remove-apply", policy)
        self.assertIn("manage-competitors.sh", policy)
        page = (ROOT / "apps/desktop/src/pages/Support.svelte").read_text(encoding="utf-8")
        self.assertIn("competitors_remove", page)
        self.assertIn("window.confirm", page)
        rust = (ROOT / "crates/chromaflow-core/src/support.rs").read_text(encoding="utf-8")
        self.assertIn("competitors_remove", rust)
        rust = (ROOT / "crates/chromaflow-core/src/conflicts.rs").read_text(encoding="utf-8")
        self.assertIn("ckb-next-daemon", rust)
        self.assertIn("is-active", rust)
        self.assertIn("is-enabled", rust)
        self.assertNotIn("/usr/lib/systemd/system", rust)
        py = (ROOT / "scripts/lib/chromaflow_competitors.py").read_text(encoding="utf-8")
        self.assertIn("_unit_live", py)
        self.assertIn("is-active", py)

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
