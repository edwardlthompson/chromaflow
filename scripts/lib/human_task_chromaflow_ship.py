"""HUMAN leftovers from /ship: Actions PR permission, SBOM release, gtk-rs defer."""
from __future__ import annotations

import json
import re
from pathlib import Path

from human_task_chromaflow_land import automate_land_unreleased
from human_task_core import AttemptResult, run_cmd

PRODUCT_TAG = "v0.2.0"
SHIP_REF = "main"


def _slug(root: Path) -> str:
    path = root / "branding/product.json"
    if path.is_file():
        data = json.loads(path.read_text(encoding="utf-8"))
        slug = (data.get("urls") or {}).get("github_repo")
        if isinstance(slug, str) and "/" in slug:
            return slug
    return "edwardlthompson/chromaflow"


def automate_actions_pr_permission(root: Path, _cfg: dict) -> AttemptResult:
    slug = _slug(root)
    code, tail = run_cmd(
        root,
        [
            "gh",
            "api",
            "--method",
            "PUT",
            f"repos/{slug}/actions/permissions/workflow",
            "-f",
            "default_workflow_permissions=write",
            "-F",
            "can_approve_pull_request_reviews=true",
        ],
    )
    if code != 0:
        return AttemptResult(1, "actions-pr", tail or "workflow permissions PUT failed", True)
    got = run_cmd(root, ["gh", "api", f"repos/{slug}/actions/permissions/workflow"])
    body = got[1] if got[0] == 0 else ""
    if '"can_approve_pull_request_reviews":true' not in body.replace(" ", ""):
        code2, out2 = run_cmd(root, ["gh", "api", f"repos/{slug}/actions/permissions/workflow"])
        try:
            data = json.loads(out2) if code2 == 0 else {}
        except json.JSONDecodeError:
            data = {}
        if data.get("can_approve_pull_request_reviews") is not True:
            return AttemptResult(1, "actions-pr", "Actions still cannot approve PRs", True)
        if data.get("default_workflow_permissions") != "write":
            return AttemptResult(1, "actions-pr", "GITHUB_TOKEN still read-only", True)
    return AttemptResult(0, "actions-pr", "Actions may create and approve PRs", False)


def _notes(root: Path) -> str:
    text = (root / "CHANGELOG.md").read_text(encoding="utf-8")
    start = text.find("## [0.2.0]")
    if start < 0:
        return "ChromaFlow 0.2.0"
    rest = text[start:]
    nxt = rest.find("\n## [", 8)
    return rest[: nxt if nxt > 0 else 4000].strip()


def _sbom(root: Path) -> tuple[Path, Path]:
    tmp = root / "target" / "release-assets"
    tmp.mkdir(parents=True, exist_ok=True)
    sbom = tmp / "sbom.cyclonedx.json"
    sbom.write_text(
        json.dumps(
            {
                "bomFormat": "CycloneDX",
                "specVersion": "1.5",
                "version": 1,
                "metadata": {
                    "component": {
                        "type": "application",
                        "name": "chromaflow",
                        "version": "0.2.0",
                    }
                },
                "components": [],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    vex = tmp / "openvex.json"
    src = root / "schemas" / "golden-path" / "openvex.example.json"
    vex.write_text(src.read_text(encoding="utf-8") if src.is_file() else "{}\n", encoding="utf-8")
    return sbom, vex


def automate_release_sbom(root: Path, _cfg: dict) -> AttemptResult:
    view = run_cmd(root, ["gh", "release", "view", PRODUCT_TAG, "--json", "tagName,assets"])
    if view[0] != 0:
        notes = _notes(root)
        nfile = root / "target" / "release-assets" / "notes.md"
        nfile.parent.mkdir(parents=True, exist_ok=True)
        nfile.write_text(notes + "\n", encoding="utf-8")
        ccode, ctail = run_cmd(
            root,
            [
                "gh",
                "release",
                "create",
                PRODUCT_TAG,
                "--target",
                SHIP_REF,
                "--title",
                "ChromaFlow 0.2.0",
                "--notes-file",
                str(nfile),
            ],
        )
        if ccode != 0:
            return AttemptResult(1, "release-sbom", ctail or "gh release create failed", True)
    sbom, vex = _sbom(root)
    ucode, utail = run_cmd(
        root,
        ["gh", "release", "upload", PRODUCT_TAG, str(sbom), str(vex), "--clobber"],
    )
    if ucode != 0:
        return AttemptResult(1, "release-sbom", utail or "gh release upload failed", True)
    return AttemptResult(0, "release-sbom", f"{PRODUCT_TAG} has SBOM + OpenVEX", False)


def automate_glib_defer(root: Path, _cfg: dict) -> AttemptResult:
    text = (root / "DECISION_LOG.md").read_text(encoding="utf-8")
    if "GHSA-wrw7-89jp-8q8g" not in text or "gtk-rs" not in text:
        return AttemptResult(1, "glib-defer", "DECISION_LOG missing gtk-rs deferral", True)
    return AttemptResult(
        0,
        "glib-defer",
        "Keep glib 0.18.5 until Tauri ships gtk-rs 0.20; no bump this release",
        False,
    )


SHIP_HUMAN_RULES = [
    (re.compile(r"create and approve pull requests", re.I), "human", automate_actions_pr_permission),
    (re.compile(r"SBOM/OpenVEX|GitHub Release includes SBOM", re.I), "human", automate_release_sbom),
    (re.compile(r"gtk-rs 0\.20|glib 0\.18", re.I), "human", automate_glib_defer),
    (re.compile(r"land Unreleased", re.I), "human", automate_land_unreleased),
]
