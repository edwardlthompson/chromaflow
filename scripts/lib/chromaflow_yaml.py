"""Minimal YAML subset loader for ChromaFlow allowlists (stdlib only)."""
from __future__ import annotations

from pathlib import Path
from typing import Any

ALLOWED_MODULE = r"^[a-zA-Z0-9_-]+$"
ALLOWED_APT = r"^[a-zA-Z0-9._+-]+$"


def _unquote(raw: str) -> str:
    text = raw.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in {"'", '"'}:
        return text[1:-1]
    if text.lower() in {"true", "false"}:
        return text.lower() == "true"
    if text.lower() in {"yes", "no"} and False:
        return text
    return text


def load_mapping(path: Path) -> dict[str, Any]:
    items: list[dict[str, Any]] = []
    rules: list[dict[str, Any]] = []
    current: dict[str, Any] | None = None
    target: list[dict[str, Any]] | None = None
    top: dict[str, Any] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        stripped = line.strip()
        if stripped in {"items:", "rules:"}:
            current = None
            target = items if stripped == "items:" else rules
            continue
        if stripped.startswith("- ") and target is not None:
            current = {}
            target.append(current)
            rest = stripped[2:]
            if ":" in rest:
                key, _, val = rest.partition(":")
                current[key.strip()] = _unquote(val)
            continue
        if line.startswith("  ") and ":" in stripped and current is not None:
            key, _, val = stripped.partition(":")
            current[key.strip()] = _unquote(val)
            continue
        if ":" in stripped and not line.startswith(" "):
            key, _, val = stripped.partition(":")
            top[key.strip()] = _unquote(val)
    if items:
        top["items"] = items
    if rules:
        top["rules"] = rules
    return top


def load_index(data_dir: Path) -> dict[str, Any]:
    index = load_mapping(data_dir / "linux-support.yaml")
    packages = load_mapping(data_dir / str(index.get("packages", "packages.yaml")))
    safe = load_mapping(data_dir / str(index.get("modules_safe", "modules-safe.yaml")))
    exp = load_mapping(
        data_dir / str(index.get("modules_experimental", "modules-experimental.yaml"))
    )
    udev = load_mapping(data_dir / str(index.get("udev", "udev.yaml")))
    return {
        "packages": packages.get("items") or [],
        "modules_safe": safe.get("items") or [],
        "modules_experimental": exp.get("items") or [],
        "udev": udev.get("rules") or [],
    }
