"""HUMAN: land Unreleased when the working tree is already committed."""
from __future__ import annotations

from pathlib import Path

from human_task_core import AttemptResult, run_cmd


def automate_land_unreleased(root: Path, _cfg: dict) -> AttemptResult:
    code, out = run_cmd(root, ["git", "status", "--porcelain"])
    if code != 0:
        return AttemptResult(1, "land-unreleased", out or "git status failed", True)
    if out.strip():
        return AttemptResult(1, "land-unreleased", "working tree still dirty", True)
    return AttemptResult(0, "land-unreleased", "Unreleased is on HEAD", False)
