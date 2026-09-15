"""Weekly health BUILD_PLAN sync must PR when main is protected."""
from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HELPER = ROOT / "scripts" / "ci-push-or-pr.sh"
WEEKLY = ROOT / ".github" / "workflows" / "weekly-health-check.yml"
SYNC = ROOT / ".github" / "workflows" / "sync-open-prs-build-plan.yml"


class CiPushOrPrTests(unittest.TestCase):
    def test_helper_opens_pr_on_protected_branch(self) -> None:
        text = HELPER.read_text(encoding="utf-8")
        self.assertIn("gh pr create", text)
        self.assertIn("chore/ci-sync-", text)
        self.assertIn("DEFAULT_BRANCH", text)
        self.assertNotIn("set_pwm", text)

    def test_weekly_health_uses_helper(self) -> None:
        weekly = WEEKLY.read_text(encoding="utf-8")
        self.assertIn("ci-push-or-pr.sh", weekly)
        self.assertIn("pull-requests: write", weekly)
        self.assertIn("sync-template-gaps-build-plan.sh", weekly)
        self.assertNotIn("git pull --rebase", weekly)
        self.assertLessEqual(weekly.count("git push"), 0)

    def test_open_prs_workflow_uses_helper(self) -> None:
        sync = SYNC.read_text(encoding="utf-8")
        self.assertIn("ci-push-or-pr.sh", sync)
        self.assertIn("pull-requests: write", sync)
        spec = (ROOT / "docs/features/ci-push-or-pr.md").read_text(encoding="utf-8")
        self.assertIn("ci-push-or-pr.sh", spec)


if __name__ == "__main__":
    unittest.main()
