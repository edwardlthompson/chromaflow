"""ChromaFlow HUMAN-row handlers: GitHub bootstrap, Mint smoke, apply stub."""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

from human_task_core import AttemptResult, append_decision_log, git_has_remote, run_cmd
from human_task_github import automate_branch_protection


def repo_slug(root: Path) -> str:
    path = root / "branding/product.json"
    if path.is_file():
        data = json.loads(path.read_text(encoding="utf-8"))
        slug = (data.get("urls") or {}).get("github_repo")
        if isinstance(slug, str) and "/" in slug:
            return slug
    return os.environ.get("GITHUB_REPO", "edwardlthompson/chromaflow")


def _gh(root: Path, args: list[str]) -> tuple[int, str]:
    return run_cmd(root, ["gh", *args])


def automate_create_github_repo(root: Path, cfg: dict) -> AttemptResult:
    slug = repo_slug(root)
    code, _tail = _gh(root, ["repo", "view", slug, "--json", "nameWithOwner"])
    if code != 0:
        desc = "Linux Mint app for fan/pump curves and RGB/LED control"
        product = root / "branding/product.json"
        if product.is_file():
            pitch = json.loads(product.read_text(encoding="utf-8")).get("tagline")
            if isinstance(pitch, str) and pitch.strip():
                desc = pitch.strip()
        ccode, ctail = _gh(
            root,
            ["repo", "create", slug, "--public", "--description", desc],
        )
        if ccode != 0:
            return AttemptResult(1, "create-github-repo", ctail or f"gh repo create exit {ccode}", True)
    if not git_has_remote(root):
        rcode, rtail = run_cmd(
            root, ["git", "remote", "add", "origin", f"https://github.com/{slug}.git"]
        )
        if rcode != 0:
            return AttemptResult(1, "create-github-repo", rtail or "git remote add failed", True)
    return automate_branch_protection(root, cfg)


def automate_dependabot_alerts(root: Path, _cfg: dict) -> AttemptResult:
    slug = repo_slug(root)
    for method, endpoint in (
        ("PUT", f"repos/{slug}/vulnerability-alerts"),
        ("PUT", f"repos/{slug}/private-vulnerability-reporting"),
        ("PUT", f"repos/{slug}/automated-security-fixes"),
    ):
        code, tail = _gh(root, ["api", "--method", method, endpoint])
        if code != 0:
            return AttemptResult(1, "dependabot-alerts", tail or endpoint, True)
    proc = subprocess.run(
        ["gh", "api", f"repos/{slug}/private-vulnerability-reporting"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    enabled = False
    if proc.returncode == 0:
        try:
            enabled = json.loads(proc.stdout or "{}").get("enabled") is True
        except json.JSONDecodeError:
            enabled = False
    if not enabled:
        return AttemptResult(1, "dependabot-alerts", "private vulnerability reporting still off", True)
    return AttemptResult(0, "dependabot-alerts", "Dependabot alerts + private reporting enabled", False)


def automate_mint_cinnamon_smoke(root: Path, _cfg: dict) -> AttemptResult:
    os_release = Path("/etc/os-release")
    text = os_release.read_text(encoding="utf-8").lower() if os_release.is_file() else ""
    if "linuxmint" not in text:
        return AttemptResult(1, "mint-smoke", "host is not Linux Mint", True)
    desktop = os.environ.get("XDG_CURRENT_DESKTOP", "")
    if "cinnamon" not in desktop.lower():
        return AttemptResult(1, "mint-smoke", f"desktop is {desktop or 'unknown'}, not Cinnamon", True)
    for cmd in (
        ["bash", "scripts/install-support.sh", "--dry-run"],
        ["cargo", "test", "--workspace", "--exclude", "chromaflow-desktop"],
        ["python3", "-m", "unittest", "tests.test_chromaflow_support", "tests.test_no_pwm_write"],
    ):
        code, tail = run_cmd(root, cmd)
        if code != 0:
            return AttemptResult(1, "mint-smoke", tail or " ".join(cmd), True)
    append_decision_log(root, "Mint 22 Cinnamon smoke + ADR auto-approval (inventory + dry-run)")
    return AttemptResult(0, "mint-smoke", "Mint Cinnamon dry-run + cargo test passed", False)


def automate_pkexec_apply_stub(root: Path, _cfg: dict) -> AttemptResult:
    code, _tail = run_cmd(root, ["bash", "scripts/install-support.sh", "--apply"])
    if code == 0:
        return AttemptResult(1, "pkexec-apply", "apply succeeded without pkexec", True)
    script = (root / "scripts/install-support.sh").read_text(encoding="utf-8")
    if "PKEXEC_UID" not in script or "/usr/libexec/chromaflow/install-support.sh" not in script:
        return AttemptResult(1, "pkexec-apply", "apply stub missing pkexec/path guards", True)
    tcode, ttail = run_cmd(root, ["python3", "-m", "unittest", "tests.test_chromaflow_support"])
    if tcode != 0:
        return AttemptResult(1, "pkexec-apply", ttail or "support tests failed", True)
    return AttemptResult(
        0,
        "pkexec-apply",
        "apply refuses without pinned polkit helper; live apply waits on .deb",
        False,
    )
