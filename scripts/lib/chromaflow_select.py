"""Select allowlist rows from probe text and already-loaded modules."""
from __future__ import annotations

import os
from pathlib import Path

ALWAYS_LOAD = frozenset({"i2c-dev"})


def module_loaded(name: str) -> bool:
    override = os.environ.get("CHROMAFLOW_LOADED_MODULES")
    if override is not None:
        names = {n.strip().replace("-", "_") for n in override.replace(",", "\n").splitlines() if n.strip()}
        return name.replace("-", "_") in names
    return Path("/sys/module", name.replace("-", "_")).is_dir()


def row_matches(row: dict, blob: str) -> bool:
    needles = [str(row[k]) for k in ("match_lspci", "match_lsusb", "match_dmi") if row.get(k)]
    if not needles:
        return False
    lower = blob.lower()
    return any(n.lower() in lower for n in needles)


def include_safe_module(row: dict, blob: str) -> bool:
    name = str(row.get("modprobe") or "")
    if name in ALWAYS_LOAD:
        return True
    if row_matches(row, blob):
        return True
    return module_loaded(name)
