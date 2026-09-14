"""OpenRGB identity extractor: detector rows only, no controller bodies."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
import sys

sys.path.insert(0, str(ROOT / "scripts" / "lib"))
from openrgb_device_index import extract_rows, write_csv  # noqa: E402


class OpenRgbIndexTests(unittest.TestCase):
    def test_extracts_hid_and_i2c_identity(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "pci_ids").mkdir()
            (root / "pci_ids" / "pci_ids.h").write_text(
                "#define NVIDIA_VEN 0x10DE\n#define NVIDIA_RTX4090_DEV 0x2684\n"
                "#define MSI_SUB_VEN 0x1462\n#define MSI_X 0x5104\n",
                encoding="utf-8",
            )
            (root / "DetectionManager.h").write_text("#define HID_PID_ANY -1\n", encoding="utf-8")
            d = root / "Controllers" / "QMK"
            d.mkdir(parents=True)
            (d / "QMKDetect.cpp").write_text(
                '#define KEYCHRON_VID 0x3434\n'
                'REGISTER_HID_DETECTOR_PU("Keychron RGB QMK/ZMK Keyboard", DetectQMKKeychronControllers, KEYCHRON_VID, HID_PID_ANY, 1, 1);\n'
                'REGISTER_I2C_PCI_DETECTOR("MSI GeForce RTX 4090 Suprim Liquid X", DetectMSIGPUv2Controllers, NVIDIA_VEN, NVIDIA_RTX4090_DEV, MSI_SUB_VEN, MSI_X, 0x68);\n',
                encoding="utf-8",
            )
            rows = extract_rows(root)
            names = {r["Name"] for r in rows}
            self.assertIn("Keychron RGB QMK/ZMK Keyboard", names)
            self.assertIn("MSI GeForce RTX 4090 Suprim Liquid X", names)
            key = next(r for r in rows if r["VID"] == "3434")
            self.assertEqual(key["PID"], "any")
            gpu = next(r for r in rows if "4090" in r["Name"])
            self.assertEqual(gpu["Type"], "I2C")
            dest = root / "out.csv"
            write_csv(rows, dest, "81bbe18")
            text = dest.read_text(encoding="utf-8")
            self.assertIn("OpenRGB detector identity", text)
            self.assertNotIn("set_pwm", text)


if __name__ == "__main__":
    unittest.main()
