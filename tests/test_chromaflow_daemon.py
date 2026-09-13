"""chromaflowd watchdog writes fake sysfs; failsafe restores enable=2."""
from __future__ import annotations

import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class DaemonFailsafeTests(unittest.TestCase):
    def test_unit_has_execstoppost(self) -> None:
        unit = (ROOT / "packaging/chromaflowd.service").read_text(encoding="utf-8")
        self.assertIn("ExecStopPost=/usr/libexec/chromaflow/pwm-failsafe.sh", unit)
        self.assertIn("chromaflow daemon --watchdog", unit)
        self.assertIn("Environment=DISPLAY=:0", unit)
        self.assertIn("Environment=XAUTHORITY=%h/.Xauthority", unit)
        sdk = (ROOT / "packaging/chromaflow-sdk.service").read_text(encoding="utf-8")
        self.assertIn("chromaflow daemon --sdk", sdk)
        self.assertNotIn("User=root", sdk)
        self.assertNotIn("set_pwm", sdk)
        self.assertNotIn("User=root", unit)
        self.assertNotIn("set_pwm", unit)

    def test_failsafe_restores_enable_not_duty(self) -> None:
        script = ROOT / "packaging/pwm-failsafe.sh"
        text = script.read_text(encoding="utf-8")
        self.assertIn("pwm*_enable=2", text)
        self.assertIn("apply=true", text)
        self.assertNotIn("set_pwm", text)
        with tempfile.TemporaryDirectory(prefix="cf-fail-") as raw:
            root = Path(raw)
            chip = root / "hwmon0"
            chip.mkdir()
            (chip / "pwm1").write_text("51\n", encoding="utf-8")
            (chip / "pwm1_enable").write_text("1\n", encoding="utf-8")
            owned = root / "owned"
            owned.write_text("hwmon0 pwm1\n", encoding="utf-8")
            env = os.environ.copy()
            env["CHROMAFLOW_HWMON_ROOT"] = str(root)
            env["CHROMAFLOW_OWNED"] = str(owned)
            proc = subprocess.run(["bash", str(script)], check=False, capture_output=True, text=True, env=env)
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertEqual((chip / "pwm1_enable").read_text(encoding="utf-8").strip(), "2")
            self.assertEqual((chip / "pwm1").read_text(encoding="utf-8").strip(), "51")
            self.assertIn("apply=true", proc.stdout)

    def test_daemon_watchdog_once_on_fake_hwmon(self) -> None:
        env = os.environ.copy()
        env["CHROMAFLOW_DAEMON_ONCE"] = "1"
        deny = subprocess.run(
            ["cargo", "run", "-q", "-p", "chromaflow-cli", "--", "daemon"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            env=env,
        )
        self.assertNotEqual(deny.returncode, 0)
        self.assertIn("--watchdog", deny.stderr + deny.stdout)
        self.assertIn("--sdk", deny.stderr + deny.stdout)
        dry = subprocess.run(
            ["cargo", "run", "-q", "-p", "chromaflow-cli", "--", "daemon", "--dry-run"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            env=env,
        )
        self.assertEqual(dry.returncode, 0, dry.stderr)
        self.assertIn("no PWM", dry.stderr)
        env["CHROMAFLOW_NO_SPAWN"] = "1"
        sdk = subprocess.run(
            ["cargo", "run", "-q", "-p", "chromaflow-cli", "--", "daemon", "--sdk"],
            check=False,
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            env=env,
        )
        self.assertEqual(sdk.returncode, 0, sdk.stderr)
        self.assertIn("no PWM", sdk.stderr)
        env.pop("CHROMAFLOW_NO_SPAWN", None)
        with tempfile.TemporaryDirectory(prefix="cf-wd-") as raw:
            root = Path(raw)
            chip = root / "hwmon0"
            chip.mkdir()
            (chip / "name").write_text("nct6775\n", encoding="utf-8")
            (chip / "pwm1").write_text("128\n", encoding="utf-8")
            (chip / "pwm1_enable").write_text("2\n", encoding="utf-8")
            (chip / "temp1_input").write_text("45000\n", encoding="utf-8")
            cfg = root / "cfg"
            cfg.mkdir()
            (cfg / "curves.json").write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "allow_zero": False,
                        "min_duty": 20,
                        "max_duty": 100,
                        "channels": [
                            {
                                "chip": "nct6775",
                                "pwm": "pwm1",
                                "dir": "hwmon0",
                                "source_chip": "nct6775",
                                "source_label": "temp1_input",
                                "enabled": True,
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            owned = root / "owned"
            env["CHROMAFLOW_HWMON_ROOT"] = str(root)
            env["CHROMAFLOW_CONFIG_HOME"] = str(cfg)
            env["CHROMAFLOW_OWNED"] = str(owned)
            env["CHROMAFLOW_CONFLICTS"] = "none"
            watch = subprocess.run(
                ["cargo", "run", "-q", "-p", "chromaflow-cli", "--", "daemon", "--watchdog"],
                check=False,
                capture_output=True,
                text=True,
                cwd=str(ROOT),
                env=env,
            )
            self.assertEqual(watch.returncode, 0, watch.stderr)
            self.assertIn("watchdog", watch.stderr)
            self.assertEqual((chip / "pwm1_enable").read_text(encoding="utf-8").strip(), "1")
            duty = int((chip / "pwm1").read_text(encoding="utf-8").strip())
            self.assertGreater(duty, 0)

    def test_schema_v2_three_channels_and_calibrate_skips_zero(self) -> None:
        recipe = (ROOT / "crates/chromaflow-core/src/pwm_recipe.rs").read_text(encoding="utf-8")
        self.assertIn("(30.0, 20)", recipe)
        self.assertIn("gauge:cpu", recipe)
        cal = (ROOT / "crates/chromaflow-core/src/pwm_calibrate.rs").read_text(encoding="utf-8")
        self.assertIn("MIN_PERCENT", cal)
        self.assertIn("take-over first", cal)
        self.assertNotIn("set_pwm", cal)
        curves = (ROOT / "crates/chromaflow-core/src/pwm_curves.rs").read_text(encoding="utf-8")
        self.assertIn("schema: 2", curves)
        self.assertIn("cooling.json", curves)
        env = os.environ.copy()
        env["CHROMAFLOW_DAEMON_ONCE"] = "1"
        with tempfile.TemporaryDirectory(prefix="cf-v2-") as raw:
            root = Path(raw)
            chip = root / "hwmon0"
            chip.mkdir()
            (chip / "name").write_text("it87952\n", encoding="utf-8")
            for n in (1, 2, 3):
                (chip / f"pwm{n}").write_text("255\n", encoding="utf-8")
                (chip / f"pwm{n}_enable").write_text("2\n", encoding="utf-8")
                (chip / f"fan{n}_input").write_text("0\n" if n < 3 else "1674\n", encoding="utf-8")
            (chip / "temp1_input").write_text("65600\n", encoding="utf-8")
            cfg = root / "cfg"
            cfg.mkdir()
            (cfg / "curves.json").write_text(
                json.dumps(
                    {
                        "schema": 2,
                        "allow_zero": False,
                        "min_duty": 20,
                        "max_duty": 100,
                        "mixes": [
                            {
                                "id": "cpu-gpu",
                                "label": "CPU+GPU",
                                "op": "max",
                                "sources": ["gauge:cpu", "gauge:gpu"],
                                "offset": 0,
                            }
                        ],
                        "names": {},
                        "calibration": {},
                        "channels": [
                            {
                                "chip": "it87952",
                                "pwm": f"pwm{n}",
                                "dir": "hwmon0",
                                "source_chip": "it87952",
                                "source_label": "temp1_input",
                                "enabled": True,
                                "curve_id": "balanced",
                                "temp_id": "hwmon:it87952/temp1_input",
                                "kind": "fan",
                                "step_up": 5,
                                "step_down": 5,
                                "hysteresis_c": 0,
                                "min_pct": 20,
                            }
                            for n in (1, 2, 3)
                        ],
                    }
                ),
                encoding="utf-8",
            )
            owned = root / "owned"
            env["CHROMAFLOW_HWMON_ROOT"] = str(root)
            env["CHROMAFLOW_CONFIG_HOME"] = str(cfg)
            env["CHROMAFLOW_OWNED"] = str(owned)
            env["CHROMAFLOW_CONFLICTS"] = "none"
            watch = subprocess.run(
                ["cargo", "run", "-q", "-p", "chromaflow-cli", "--", "daemon", "--watchdog"],
                check=False,
                capture_output=True,
                text=True,
                cwd=str(ROOT),
                env=env,
            )
            self.assertEqual(watch.returncode, 0, watch.stderr)
            for n in (1, 2, 3):
                self.assertEqual((chip / f"pwm{n}_enable").read_text(encoding="utf-8").strip(), "1")
                duty = int((chip / f"pwm{n}").read_text(encoding="utf-8").strip())
                self.assertGreater(duty, 0)

    def test_wrapper_is_executable_script(self) -> None:
        path = ROOT / "packaging/chromaflowd"
        text = path.read_text(encoding="utf-8")
        self.assertIn("chromaflow daemon", text)
        self.assertIn("--watchdog", text)
        mode = path.stat().st_mode
        self.assertTrue(mode & stat.S_IXUSR)

    def test_pwm_acl_chmods_without_writing_duty(self) -> None:
        script = ROOT / "packaging/pwm-acl.sh"
        text = script.read_text(encoding="utf-8")
        self.assertIn("plugdev", text)
        self.assertIn("0660", text)
        self.assertNotIn("set_pwm", text)
        with tempfile.TemporaryDirectory(prefix="cf-acl-") as raw:
            root = Path(raw)
            chip = root / "hwmon0"
            chip.mkdir()
            pwm = chip / "pwm1"
            pwm.write_text("77\n", encoding="utf-8")
            enable = chip / "pwm1_enable"
            enable.write_text("2\n", encoding="utf-8")
            env = os.environ.copy()
            env["CHROMAFLOW_HWMON_ROOT"] = str(root)
            proc = subprocess.run(["bash", str(script)], check=False, capture_output=True, text=True, env=env)
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertEqual(pwm.read_text(encoding="utf-8").strip(), "77")
            self.assertEqual(enable.read_text(encoding="utf-8").strip(), "2")
            self.assertTrue(pwm.stat().st_mode & 0o020)

    def test_pwm_daemon_enable_is_user_unit(self) -> None:
        src = (ROOT / "crates/chromaflow-core/src/pwm_daemon.rs").read_text(encoding="utf-8")
        self.assertIn("enable", src)
        self.assertIn("--now", src)
        self.assertIn("chromaflowd.service", src)
        self.assertIn("--user", src)
        self.assertNotIn("set_pwm", src)


if __name__ == "__main__":
    unittest.main()
