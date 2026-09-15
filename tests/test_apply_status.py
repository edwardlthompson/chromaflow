"""Lighting apply status hides protocol broadcast strings."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class ApplyStatusTests(unittest.TestCase):
    def test_strips_broadcast_hex(self) -> None:
        lighting = (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8")
        self.assertIn("applyStatus", lighting)
        self.assertIn("applyStatus.js", lighting)
        text = (ROOT / "apps/desktop/src/lib/applyStatus.js").read_text(encoding="utf-8")
        self.assertIn("broadcast [0-9A-Fa-f]", text)
        self.assertIn("extra[1] || okText", text)
        self.assertNotIn("set_pwm", text)
        spec = (ROOT / "docs/features/desktop-chrome.md").read_text(encoding="utf-8")
        self.assertIn("applyStatus", spec)
        profiles = (ROOT / "apps/desktop/src/pages/Profiles.svelte").read_text(encoding="utf-8")
        self.assertNotIn("JSON.stringify", profiles)
        self.assertIn("fc-form", profiles)
        css = (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8")
        self.assertIn("Ubuntu", css)
        self.assertIn("fc-spin", css)
        self.assertIn("[aria-busy=\"true\"]", css)
        self.assertNotIn("cursor: wait", css)
        self.assertIn(":focus-visible", css)
        self.assertIn(".rail-label", css)
        app = (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8")
        self.assertIn("fc-pill", app)
        self.assertNotIn("Showing {source}", app)


if __name__ == "__main__":
    unittest.main()
