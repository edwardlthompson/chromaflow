"""GitHub Pages site uses workflow source, not a 404."""
from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

LIB = Path(__file__).resolve().parent.parent / "scripts" / "lib"
if str(LIB) not in sys.path:
    sys.path.insert(0, str(LIB))

from github_pages_enable import dispatch_pages, enable_pages  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent


class _FakeGh:
    def __init__(self, mapping: dict[tuple[str, ...], tuple[int, str]]) -> None:
        self.mapping = mapping
        self.calls: list[tuple[str, ...]] = []

    def __call__(self, _root: Path, args: list[str]) -> tuple[int, str]:
        key = tuple(args)
        self.calls.append(key)
        return self.mapping.get(key, (1, "unexpected " + " ".join(args)))


class GitHubPagesEnableTests(unittest.TestCase):
    def test_already_enabled_workflow(self) -> None:
        fake = _FakeGh(
            {
                ("api", "repos/{owner}/{repo}/pages"): (
                    0,
                    json.dumps({"build_type": "workflow", "html_url": "https://ex.github.io/x/"}),
                )
            }
        )
        code, msg, created = enable_pages(ROOT, runner=fake)
        self.assertEqual(code, 0)
        self.assertFalse(created)
        self.assertIn("already", msg)

    def test_creates_workflow_source(self) -> None:
        fake = _FakeGh(
            {
                ("api", "repos/{owner}/{repo}/pages"): (1, "Not Found"),
                (
                    "api",
                    "--method",
                    "POST",
                    "repos/{owner}/{repo}/pages",
                    "-f",
                    "build_type=workflow",
                ): (0, json.dumps({"status": "built"})),
            }
        )
        code, msg, created = enable_pages(ROOT, runner=fake)
        self.assertEqual(code, 0)
        self.assertTrue(created)
        self.assertIn("created", msg)

    def test_legacy_source_fails(self) -> None:
        fake = _FakeGh(
            {
                ("api", "repos/{owner}/{repo}/pages"): (
                    0,
                    json.dumps({"build_type": "legacy"}),
                )
            }
        )
        code, _msg, created = enable_pages(ROOT, runner=fake)
        self.assertEqual(code, 1)
        self.assertFalse(created)

    def test_dispatch_and_docs(self) -> None:
        fake = _FakeGh({("workflow", "run", "pages.yml"): (0, "")})
        self.assertEqual(dispatch_pages(ROOT, runner=fake)[0], 0)
        spec = (ROOT / "docs/features/github-pages-enable.md").read_text(encoding="utf-8")
        self.assertIn("enable-github-pages", spec)
        weekly = (ROOT / "scripts/enable-github-pages.sh").read_text(encoding="utf-8")
        self.assertIn("github_pages_enable.py", weekly)


if __name__ == "__main__":
    unittest.main()
