"""Profile schema is names only; Support filter is machine-specific."""
from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "scripts" / "lib"
if str(LIB) not in sys.path:
    sys.path.insert(0, str(LIB))

from chromaflow_select import include_safe_module  # noqa: E402
from chromaflow_support import build_plan  # noqa: E402


class ProfileAndSelectTests(unittest.TestCase):
    def test_unmatched_row_needs_loaded_or_i2c_dev(self) -> None:
        blob = "ASUS ROG\nIntel SMBus"
        os.environ["CHROMAFLOW_LOADED_MODULES"] = ""
        self.assertTrue(include_safe_module({"modprobe": "i2c-dev"}, blob))
        self.assertTrue(include_safe_module({"modprobe": "asus-wmi", "match_dmi": "ASUS"}, blob))
        self.assertTrue(
            include_safe_module({"modprobe": "gigabyte_wmi", "match_dmi": "Gigabyte"}, "Gigabyte X570S")
        )
        self.assertFalse(include_safe_module({"modprobe": "nct6775"}, blob))
        os.environ["CHROMAFLOW_LOADED_MODULES"] = "nct6775"
        self.assertTrue(include_safe_module({"modprobe": "nct6775"}, blob))

    def test_modules_load_d_plan_is_dry_run(self) -> None:
        os.environ["CHROMAFLOW_LSPCI"] = "Intel SMBus"
        os.environ["CHROMAFLOW_LSUSB"] = ""
        os.environ["CHROMAFLOW_DMI"] = "ASUS"
        os.environ["CHROMAFLOW_LOADED_MODULES"] = ""
        plan = build_plan(ROOT / "data", advanced=False)
        self.assertTrue(plan["ok"])
        self.assertFalse(plan["would_modules_load_d"]["apply"])
        self.assertEqual(
            plan["would_modules_load_d"]["path"],
            "/etc/modules-load.d/chromaflow.conf",
        )

    def test_profiles_apply_maps_looks_without_pwm_write(self) -> None:
        import subprocess

        out = subprocess.check_output(
            [
                "node",
                "--input-type=module",
                "-e",
                "import { rgbLook, withCurveSet, needsZeroAsk, canApplyNames, takeoverFile } from './apps/desktop/src/lib/profilesApply.js';"
                "if (rgbLook('off').color !== '#000000') throw new Error('off');"
                "if (rgbLook('solid').mode !== 'Solid Color') throw new Error('solid');"
                "if (canApplyNames('quiet', 'nope') || !canApplyNames('quiet', 'solid')) throw new Error('names');"
                "const f = withCurveSet({ channels: [{ enabled: false, curve_id: 'x', min_pct: 20 }] }, 'quiet');"
                "if (!f.channels[0].enabled || f.channels[0].curve_id !== 'quiet') throw new Error('curve');"
                "if (!needsZeroAsk({ channels: [{ enabled: true, min_pct: 0 }] })) throw new Error('zero');"
                "const built = takeoverFile({ channels: [] }, { hwmon: [], gpu_fans: [] }, 'balanced');"
                "if (!Array.isArray(built.channels)) throw new Error('file');",
            ],
            cwd=ROOT,
            text=True,
        )
        self.assertEqual(out.strip(), "")
        src = (ROOT / "apps/desktop/src/lib/profilesApply.js").read_text(encoding="utf-8")
        self.assertNotIn("set_pwm", src)
        text = (ROOT / "crates/chromaflow-core/src/profiles.rs").read_text(encoding="utf-8")
        self.assertIn("schema: u32", text)
        self.assertIn("curve_set", text)
        self.assertNotIn("set_pwm", text)
        self.assertIn(".config", text)

    def test_config_home_override_is_documented(self) -> None:
        text = (ROOT / "crates/chromaflow-core/src/profiles.rs").read_text(encoding="utf-8")
        self.assertIn("CHROMAFLOW_CONFIG_HOME", text)
        with tempfile.TemporaryDirectory() as tmp:
            self.assertTrue(Path(tmp).is_dir())


if __name__ == "__main__":
    unittest.main()
