"""ChromaFlow BUILD_PLAN HUMAN rows have automation handlers."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

LIB = Path(__file__).resolve().parent.parent / "scripts" / "lib"
if str(LIB) not in sys.path:
    sys.path.insert(0, str(LIB))

from human_task_chromaflow import automate_pkexec_apply_stub  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent

TASKS = (
    "Create GitHub repo and run `scripts/setup-github-repo.sh` (`gh` admin)",
    "Enable Dependabot alerts and private vulnerability reporting",
    "Bookmark `docs/help/BATCH_COMMANDS.md`",
    "Approve ADRs and smoke on a Mint 21/22 Cinnamon machine",
    "Live pkexec `--apply` on a VM after `.deb` helper exists (not this sprint)",
)


class ChromaflowHumanAutomationTests(unittest.TestCase):
    def test_rules_match_open_rows(self) -> None:
        from human_task_automation import HUMAN_RULES

        for task in TASKS:
            matched = any(pattern.search(task) for pattern, _kind, _handler in HUMAN_RULES)
            self.assertTrue(matched, f"no rule for {task}")

    def test_pkexec_apply_refuses(self) -> None:
        result = automate_pkexec_apply_stub(ROOT, {})
        self.assertEqual(result.exit_code, 0, result.reason)
        self.assertFalse(result.backlog)


if __name__ == "__main__":
    unittest.main()
