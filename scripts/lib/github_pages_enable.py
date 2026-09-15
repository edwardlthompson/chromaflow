"""Enable GitHub Pages with Actions (workflow) as the source."""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def run_gh(root: Path, args: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        ["gh", *args],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode, (proc.stdout or proc.stderr or "").strip()


def enable_pages(root: Path, *, runner=run_gh) -> tuple[int, str, bool]:
    """Return (code, message, created)."""
    code, body = runner(root, ["api", "repos/{owner}/{repo}/pages"])
    if code == 0:
        try:
            data = json.loads(body)
        except json.JSONDecodeError:
            data = {}
        if not isinstance(data, dict):
            return 1, "unexpected Pages payload", False
        if data.get("build_type") == "legacy":
            return 1, "Pages is branch/folder source; switch to GitHub Actions", False
        html = data.get("html_url") or "ok"
        return 0, f"Pages already enabled ({html})", False
    code, body = runner(
        root,
        [
            "api",
            "--method",
            "POST",
            "repos/{owner}/{repo}/pages",
            "-f",
            "build_type=workflow",
        ],
    )
    if code != 0:
        return code, body[-400:], False
    return 0, "created Pages site with workflow source", True


def dispatch_pages(root: Path, *, runner=run_gh) -> tuple[int, str]:
    return runner(root, ["workflow", "run", "pages.yml"])


def main() -> int:
    code, msg, _created = enable_pages(ROOT)
    print(msg)
    if code != 0:
        return 1
    dcode, dmsg = dispatch_pages(ROOT)
    if dcode != 0:
        print(f"NOTE: could not dispatch pages.yml: {dmsg[-300:]}")
        return 0
    print(dmsg or "dispatched pages.yml")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
