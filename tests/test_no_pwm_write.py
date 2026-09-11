"""Product crates must not grow a PWM write API this milestone."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUST_ROOTS = (ROOT / "crates", ROOT / "apps/desktop/src-tauri")
FORBIDDEN = ("set_pwm", "write_pwm", "pwm_write")


class NoPwmWriteTests(unittest.TestCase):
    def test_no_pwm_write_symbols(self) -> None:
        hits = []
        for root in RUST_ROOTS:
            for path in root.rglob("*.rs"):
                text = path.read_text(encoding="utf-8")
                for needle in FORBIDDEN:
                    if needle in text:
                        hits.append(f"{path.relative_to(ROOT)}:{needle}")
        self.assertEqual(hits, [])

    def test_cli_help_mentions_readonly(self) -> None:
        main = (ROOT / "crates/chromaflow-cli/src/main.rs").read_text(encoding="utf-8")
        self.assertIn("never writes PWM", main)
        self.assertIn("refusing apply/PWM", main)


if __name__ == "__main__":
    unittest.main()
