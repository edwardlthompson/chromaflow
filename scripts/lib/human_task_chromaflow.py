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


def automate_tauri_dev_packages(root: Path, _cfg: dict) -> AttemptResult:
    pkgs = [
        "libwebkit2gtk-4.1-dev",
        "libgtk-3-dev",
        "libayatana-appindicator3-dev",
        "librsvg2-dev",
        "libssl-dev",
    ]
    missing = [
        name
        for name in pkgs
        if "install ok installed"
        not in subprocess.run(
            ["dpkg-query", "-W", "-f=${Status}", name],
            capture_output=True,
            text=True,
            check=False,
        ).stdout
    ]
    if not missing:
        return AttemptResult(0, "tauri-dev-apt", "WebKit/GTK dev packages already installed", False)
    apt = ["apt-get", "install", "-y", "--no-install-recommends", *missing]
    for wrap in (["sudo", "-n"], ["pkexec"]):
        code, tail = run_cmd(root, [*wrap, *apt])
        if code == 0:
            append_decision_log(root, "Installed Tauri GTK/WebKit dev packages via " + wrap[0])
            return AttemptResult(0, "tauri-dev-apt", "installed " + " ".join(missing), False)
    return AttemptResult(1, "tauri-dev-apt", tail or "apt needs polkit/sudo password", True)


from human_task_chromaflow_ship import (
    automate_actions_pr_permission,
    automate_glib_defer,
    automate_release_sbom,
)
from human_task_chromaflow_live import (
    automate_openrgb_server,
    automate_pkexec_apply_stub,
    automate_pwm_live_smoke,
    automate_pwm_takeover,
)

